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

import { OnChanges, OnInit, SimpleChanges } from "@angular/core";
import { Component, Input } from "@angular/core";
import { TauriService } from "src/app/shared/services/tauri.service";
import { UserPullRequestsService } from "src/app/shared/services/user-pull-requests.service";
import { PRTableEntry, TrackedPRs } from "src/app/shared/types";

@Component({
  selector: "ghd-pull-requests-card",
  templateUrl: "./pull-requests-card.component.html",
  styleUrls: ["./pull-requests-card.component.scss"],
})
export class PullRequestsCardComponent implements OnInit, OnChanges {
  @Input()
  public prs: TrackedPRs = { toView: [], viewed: [], len: 0 };

  @Input()
  public login?: string;

  public isCollapsed: boolean = true;
  public hasLogin: boolean = false;

  public markingViewed: boolean = false;
  public markingArchived: boolean = false;

  public recentHasClosed: boolean = false;
  public viewedHasClosed: boolean = false;

  public constructor(
    private tauriSvc: TauriService,
    private prsSvc: UserPullRequestsService,
  ) {}

  public ngOnInit(): void {
    this.hasLogin = !!this.login;
    if (!this.hasLogin) {
      console.error("Component missing required user login!");
    }
    this.updateState();
  }

  public ngOnChanges(changes: SimpleChanges): void {
    if ("prs" in changes) {
      this.updateState();
    }
  }

  private updateState(): void {
    this.prs.toView.forEach((entry: PRTableEntry) => {
      if (entry.state !== "open") {
        this.recentHasClosed = true;
      }
    });
    this.prs.viewed.forEach((entry: PRTableEntry) => {
      if (entry.state !== "open") {
        this.viewedHasClosed = true;
      }
    });
  }

  /**
   * Marks PRs as viewed, either all of them, or just those that have been
   * closed.
   *
   * @param all Whether we should mark all PRs viewed, or just those that have
   * been closed.
   */
  public markViewed(all: boolean = false): void {
    if (!this.hasLogin) {
      console.assert(false, "Should not reach this!");
      return;
    }

    this.markingViewed = true;

    let prlst: number[] = [];
    this.prs.toView.forEach((entry: PRTableEntry) => {
      if (all || entry.state !== "open") {
        prlst.push(entry.id);
      }
    });

    if (prlst.length > 0) {
      this.tauriSvc
        .markManyPullRequestsViewed(prlst)
        .then(() => {
          this.prsSvc.updateUser(this.login!).then(() => {});
        })
        .catch(() => {
          console.error("Unable to set PR list as viewed: ", prlst);
        })
        .finally(() => {
          this.markingViewed = false;
        });
    } else {
      this.markingViewed = false;
    }
  }

  /**
   * Marks viewed PRs as archived, either all of them, or just those that have
   * been closed.
   *
   * @param all Whether we should mark all PRs as archived, or just those that
   * have been closed.
   */
  public markArchived(all: boolean = false): void {
    if (!this.hasLogin) {
      console.assert(false, "Should not reach this!");
      return;
    }

    this.markingArchived = true;

    let prlst: number[] = [];
    this.prs.viewed.forEach((entry: PRTableEntry) => {
      if (all || entry.state !== "open") {
        prlst.push(entry.id);
      }
    });

    if (prlst.length > 0) {
      this.tauriSvc
        .archiveIssueMany(prlst)
        .then(() => {
          this.prsSvc.updateUser(this.login!).then(() => {});
        })
        .catch(() => {
          console.error("Unable to archive PR list: ", prlst);
        })
        .finally(() => {
          this.markingArchived = false;
        });
    } else {
      this.markingArchived = false;
    }
  }
}
