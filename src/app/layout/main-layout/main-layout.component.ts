import { Component, OnInit } from "@angular/core";
import { TauriService } from "src/app/shared/services/tauri.service";

@Component({
  selector: "ghd-main-layout",
  templateUrl: "./main-layout.component.html",
  styleUrls: ["./main-layout.component.scss"],
})
export class MainLayoutComponent implements OnInit {
  public version: string = "";

  public constructor(private tauriSvc: TauriService) {}

  public ngOnInit(): void {
    this.tauriSvc.getVersion().then((v) => {
      this.version = v;
    });
  }
}
