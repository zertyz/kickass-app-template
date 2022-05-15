import { ComponentFixture, TestBed } from '@angular/core/testing';

import { RestServiceExplorerPage } from './rest-service-explorer-page.component';

describe('RestServiceComponent', () => {
  let component: RestServiceExplorerPage;
  let fixture: ComponentFixture<RestServiceExplorerPage>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ RestServiceExplorerPage ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(RestServiceExplorerPage);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
