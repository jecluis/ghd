import { ComponentFixture, TestBed } from "@angular/core/testing";

import { PRStateComponent } from "./pr-state.component";

describe("PRStateComponent", () => {
  let component: PRStateComponent;
  let fixture: ComponentFixture<PRStateComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [PRStateComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(PRStateComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
