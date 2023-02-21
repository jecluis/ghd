import { ComponentFixture, TestBed } from "@angular/core/testing";

import { PullRequestsWidgetComponent } from "./pull-requests-widget.component";

describe("PullRequestsWidgetComponent", () => {
  let component: PullRequestsWidgetComponent;
  let fixture: ComponentFixture<PullRequestsWidgetComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [PullRequestsWidgetComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(PullRequestsWidgetComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
