import { ComponentFixture, TestBed } from "@angular/core/testing";

import { PullRequestsTableComponent } from "./pull-requests-table.component";

describe("PullRequestsTableComponent", () => {
  let component: PullRequestsTableComponent;
  let fixture: ComponentFixture<PullRequestsTableComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [PullRequestsTableComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(PullRequestsTableComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
