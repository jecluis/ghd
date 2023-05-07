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
import { NgbOffcanvas, NgbOffcanvasRef } from "@ng-bootstrap/ng-bootstrap";
import { TauriService } from "src/app/shared/services/tauri.service";
import { UserPullRequestsService } from "src/app/shared/services/user-pull-requests.service";
import { PRTableEntry } from "src/app/shared/types";
import { PullRequestDetailsComponent } from "../pull-request-details/pull-request-details.component";

@Component({
  selector: "ghd-pull-requests-table",
  templateUrl: "./pull-requests-table.component.html",
  styleUrls: ["./pull-requests-table.component.scss"],
})
export class PullRequestsTableComponent implements OnInit {
  @Input()
  public login!: string;

  @Input()
  public entries!: PRTableEntry[];

  @Input()
  public recents: boolean = true;

  public isMarkingSomething = false;
  public markingViewed?: number;
  public markingArchived?: number;

  public constructor(
    private tauriSvc: TauriService,
    private prsSvc: UserPullRequestsService,
    private offcanvas: NgbOffcanvas,
  ) {}

  public ngOnInit(): void {}

  public markViewed(pr: PRTableEntry): void {
    this.isMarkingSomething = true;
    this.markingViewed = pr.id;
    this.tauriSvc
      .markPullRequestViewed(pr.id)
      .then(() => {
        this.prsSvc.updateUser(this.login).then(() => {
          this.isMarkingSomething = false;
          this.markingViewed = undefined;
        });
      })
      .catch(() => {
        console.error(`unable to mark PR id ${pr.id} as viewed`);
      });
  }

  public markArchived(pr: PRTableEntry): void {
    this.isMarkingSomething = true;
    this.markingArchived = pr.id;
    this.tauriSvc
      .archiveIssue(pr.id)
      .then(() => {
        this.prsSvc.updateUser(this.login).then(() => {
          this.isMarkingSomething = false;
          this.markingArchived = undefined;
        });
      })
      .catch((err) => {
        console.error(`unable to mark PR id ${pr.id} as archived: ${err}`);
      });
  }

  public openDetail(pr: PRTableEntry): void {
    let ref: NgbOffcanvasRef = this.offcanvas.open(
      PullRequestDetailsComponent,
      {
        position: "bottom",
        panelClass: "ghd-pr-details-offcanvas",
      },
    );
    ref.componentInstance.id = pr.id;
  }
}
