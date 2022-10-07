import {Component, Input} from '@angular/core';
import {HttpClient, HttpErrorResponse} from "@angular/common/http";
import {throwError} from 'rxjs';
import {catchError, retry} from 'rxjs/operators';
import {environment} from '../../environments/environment';
import {UntypedFormBuilder, UntypedFormGroup, Validators} from '@angular/forms';
import {FieldDataType, FieldPresentationType, Form, FormField, HttpMethod} from './form';


@Component({
  selector: 'app-explorer-form',
  templateUrl: './explorer-form.component.html',
  styleUrls: ['./explorer-form.component.css']
})
export class ExplorerFormComponent {

  environment = environment;

  @Input()
  form: Form = {
    service: "<<service>>",
    method: "<<method>>",
    form_title: "<<form_title>>",
    service_description: "<<service_description>>",
    submit_label: '<<submit_label>>',
    fields: [],
  };


  // raw FormGroup info -- kept separate for debugging
  formGroupModel: {[p: string]: any} = {};

  // field validators -- to be initialized after @Input parameters
  formGroup: UntypedFormGroup = this.fb.group(this.formGroupModel);

  backendData: any = {};


  constructor(private fb: UntypedFormBuilder, private httpClient: HttpClient) {}

  ngOnInit(): void {
    this.formGroupModel = this.form.fields
      .reduce((prev: {[p: string]: any}, fieldRow: FormField[])  => {
        let next = prev;
        fieldRow.forEach((field: FormField) => {
          if (field.dataType == "Text" && field.lowerBound != null && field.upperBound != null) {
            // min & max length for text fields
            next[field.name] = [null, Validators.compose([
              Validators.required,
              Validators.minLength(field.lowerBound),
              Validators.maxLength(field.upperBound)
            ])]
          } else if (field.dataType == "Number" && field.lowerBound != null && field.upperBound != null) {
            // min & max length for text fields
            next[field.name] = [null, Validators.compose([
              Validators.required,
              Validators.min(field.lowerBound),
              Validators.max(field.upperBound)
            ])]
          } else if (field.presentationType == "RadioButtons" && field.possibleValues != null && field.required) {
            // required radio buttons comes with the first value pre-selected
            next[field.name] = [field.possibleValues[0].value, Validators.required];
          } else if (field.presentationType == "CheckBox") {
            // checkboxes default to unchecked
            next[field.name] = [false];
          } else if (field.required) {
            // field accepts any value other than empty
            next[field.name] = [null, Validators.required];
          } else {
            // field is not required
            next[field.name] = null;
          }
        });
        return next;
      }, {});

    this.formGroup = this.fb.group(this.formGroupModel);
  }

  onSubmit(): void {
    let resolvedService = this.resolveService();
    let resolvedDataToSend = this.resolveDataToSend();
    if (environment.http_debug) {
      let message = 'About to send the form \'' + JSON.stringify(this.formGroup.value) + '\' to service \''+this.form.service+'\' via '+this.form.method;
      if (resolvedService != this.form.service) {
        message = message + " => '"+resolvedService+"'";
      }
      alert(message);
    }
    if (this.form.method == "POST") {
      // POST
      this.httpClient.post<any>(resolvedService, resolvedDataToSend)
        .pipe(
          retry(3),
          catchError(this.handleHttpError)
        )
        .subscribe(data => this.backendData = data);
    } else if (resolvedDataToSend == null) {
      // REST
      this.httpClient.get<any>(resolvedService)
        .pipe(
          retry(3),
          catchError(this.handleHttpError)
        )
        .subscribe(data => this.backendData = data);
    } else {
      // GET
      this.httpClient.get<any>(resolvedService, resolvedDataToSend)
        .pipe(
          retry(3),
          catchError(this.handleHttpError)
        )
        .subscribe(data => this.backendData = data);
    }

  }

  private resolveService(): string {
    let resolvedService = this.form.service;
    if (this.form.method == "REST") {
      Object.keys(this.formGroup.value).forEach(formFieldName => resolvedService = resolvedService.replace("{"+formFieldName+"}", this.formGroup.value[formFieldName]));
    }
    return resolvedService;
  }

  private resolveDataToSend(): any {
    if (this.form.method == "REST") {
      return null;
    } else if (this.form.method == "GET") {
      return {
        //headers?: HttpHeaders | {[header: string]: string | string[]},
        //observe?: 'body' | 'events' | 'response',
        params: this.formGroup.value,
        //reportProgress?: boolean,
        //responseType?: 'arraybuffer'|'blob'|'json'|'text',
        //withCredentials?: boolean,
      };
    } else if (this.form.method == "POST") {
      return this.formGroup.value;
    }
  }

  private handleHttpError(error: HttpErrorResponse) {
    let errorMessage;
    if (error.status === 0) {
      errorMessage = 'Could not contact backend: ' + error.error;
    } else {
      errorMessage = `Backend returned code ${error.status}, body was: ` + error.error;
    }
    console.error(errorMessage);
    this.backendData = {error: errorMessage};
    alert('Error making the HTTP request: ' + errorMessage);
    return throwError(errorMessage);
  }

}
