
export interface Form {
  service: string,
  method: HttpMethod | string,
  form_title: string,
  service_description: string,
  submit_label: string,
  fields: FormField[][],
}

export interface FormField {
  label: string,
  name: string,
  dataType: FieldDataType | string,
  presentationType: FieldPresentationType | string,
  required?: boolean,
  helperText: string,
  possibleValues?: {text: string, value: string}[],
  /** for 'Number' fields, this means the lower value possible; for 'Text' fields, the minimum string */
  lowerBound?: number,
  /** for 'Number' fields, this means the greatest value possible; for 'Text' fields, the maximum string */
  upperBound?: number,
}

export enum FieldDataType {
  Number,
  Text,
  Boolean,
}

export enum FieldPresentationType {
  Input,
  SteppedInput,
  Area,
  Date,
  DateTime,
  Toggle,
  ComboBox,
  ListBox,
  CheckBox,
  RadioButtons,
}

export enum HttpMethod {
  GET,
  POST,
  REST,
}
