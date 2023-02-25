import { ComponentFixture, TestBed } from "@angular/core/testing";

import { RepoNameComponent } from "./repo-name.component";

describe("RepoNameComponent", () => {
  let component: RepoNameComponent;
  let fixture: ComponentFixture<RepoNameComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [RepoNameComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(RepoNameComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
