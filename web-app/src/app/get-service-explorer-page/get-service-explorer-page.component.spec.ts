import { ComponentFixture, TestBed } from '@angular/core/testing';

import { GetServiceExplorerPage } from './get-service-explorer-page.component';

describe('GetServiceComponent', () => {
  let component: GetServiceExplorerPage;
  let fixture: ComponentFixture<GetServiceExplorerPage>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ GetServiceExplorerPage ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(GetServiceExplorerPage);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
