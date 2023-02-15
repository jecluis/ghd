// Copyright 2023 Joao Eduardo Luis <joao@abysmo.io>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import { Component, Input, NgZone, OnDestroy, OnInit } from "@angular/core";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "src/app/shared/services/tauri.service";
import { GithubUser } from "src/app/shared/types";

@Component({
  selector: "ghd-dashboard-view",
  templateUrl: "./dashboard-view.component.html",
  styleUrls: ["./dashboard-view.component.scss"],
})
export class DashboardViewComponent
  implements OnInit, OnDestroy, TauriEventListener
{
  @Input()
  public user!: GithubUser;

  public iterationN = 0;

  public constructor(private zone: NgZone, private tauriSvc: TauriService) {}

  public ngOnInit(): void {
    this.tauriSvc.register(TauriService.events.ITERATION, this);
  }

  public ngOnDestroy(): void {
    this.tauriSvc.unregister(TauriService.events.ITERATION, this);
  }

  public getListenerID(): string {
    return "dashboard-view-listener";
  }

  public handleEvent(event: TauriListenerEvent): void {
    const evname = event.name;
    if (evname === TauriService.events.ITERATION) {
      this.zone.run(() => {
        this.iterationN = <number>event.payload;
      });
    }
  }
}
