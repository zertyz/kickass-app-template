import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PostServiceExplorerPage } from './post-service-explorer-page.component';

describe('PostServiceComponent', () => {
  let component: PostServiceExplorerPage;
  let fixture: ComponentFixture<PostServiceExplorerPage>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ PostServiceExplorerPage ]
    })
    .compileComponents();
  });

  beforeEach(() => {
    fixture = TestBed.createComponent(PostServiceExplorerPage);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
