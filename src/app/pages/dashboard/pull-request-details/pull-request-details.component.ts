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

import { Component, Input, OnInit } from "@angular/core";
import { TauriService } from "src/app/shared/services/tauri.service";
import { PullRequestInfo } from "src/app/shared/types";

@Component({
  selector: "ghd-pull-request-details",
  templateUrl: "./pull-request-details.component.html",
  styleUrls: ["./pull-request-details.component.scss"],
})
export class PullRequestDetailsComponent implements OnInit {
  @Input()
  public id?: number;

  public hasID: boolean = false;
  public hasError: boolean = false;
  public hasDetails: boolean = false;
  public details?: PullRequestInfo;

  public constructor(private tauriSvc: TauriService) {}

  public ngOnInit(): void {
    this.hasID = !!this.id;
    if (!this.hasID) {
      console.assert(false, "ID not specified!");
      return;
    }
    this.tauriSvc
      .getPullRequestInfo(this.id!)
      .then((res: PullRequestInfo) => {
        this.details = res;
        this.hasDetails = true;
      })
      .catch(() => {
        this.hasError = true;
      });
  }
}
