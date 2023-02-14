import { ComponentFixture, TestBed } from "@angular/core/testing";

import { TrackUserModalComponent } from "./track-user-modal.component";

describe("TrackUserModalComponent", () => {
  let component: TrackUserModalComponent;
  let fixture: ComponentFixture<TrackUserModalComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [TrackUserModalComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(TrackUserModalComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it("should create", () => {
    expect(component).toBeTruthy();
  });
});
