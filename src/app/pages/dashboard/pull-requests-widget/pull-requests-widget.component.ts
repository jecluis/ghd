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

import { Component, OnInit, OnDestroy, Input, NgZone } from "@angular/core";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "src/app/shared/services/tauri.service";
import { GithubUser, PullRequestEntry } from "src/app/shared/types";

type PRTableEntry = {
  id: number;
  number: number;
  title: string;
  repo: string;
  state: string;
  lastUpdate: string;
};

@Component({
  selector: "ghd-pull-requests-widget",
  templateUrl: "./pull-requests-widget.component.html",
  styleUrls: ["./pull-requests-widget.component.scss"],
})
export class PullRequestsWidgetComponent
  implements OnInit, OnDestroy, TauriEventListener
{
  @Input()
  public user!: GithubUser;

  public toViewPRs: PRTableEntry[] = [];
  public viewedPRs: PRTableEntry[] = [];

  public constructor(private zone: NgZone, private tauriSvc: TauriService) {}

  public ngOnInit(): void {
    this.tauriSvc.register(TauriService.events.USER_DATA_UPDATE, this);
    this.updateUser();
  }

  public ngOnDestroy(): void {
    this.tauriSvc.unregister(TauriService.events.USER_DATA_UPDATE, this);
  }

  public getListenerID(): string {
    return "pr-widget-listener";
  }

  public handleEvent(event: TauriListenerEvent): void {
    if (event.name === TauriService.events.USER_DATA_UPDATE) {
      console.debug(`received user data update for user '${this.user.login}'`);
      this.zone.run(() => {
        this.updateUser();
      });
    }
  }

  public markViewed(pr: PRTableEntry): void {
    this.tauriSvc
      .markPullRequestViewed(pr.id)
      .then(() => {
        this.updateUser();
      })
      .catch(() => {
        console.error(`unable to mark PR id ${pr.id} as viewed`);
      });
  }

  private updateUser(): void {
    this.tauriSvc
      .getPullRequestsByLogin(this.user.login)
      .then((res: PullRequestEntry[]) => {
        this.processPRs(res);
      })
      .catch(() => {
        console.error("unable to obtain pull requests for ", this.user.login);
      });
  }

  private processPRs(prs: PullRequestEntry[]): void {
    let toView: PRTableEntry[] = [];
    let viewed: PRTableEntry[] = [];
    prs.forEach((pr: PullRequestEntry) => {
      let entry = {
        id: pr.id,
        number: pr.number,
        title: pr.title,
        repo: `${pr.repo_owner}/${pr.repo_name}`,
        state: pr.state,
        lastUpdate: "?? ago",
      };
      if (!!pr.last_viewed && pr.last_viewed >= pr.updated_at) {
        viewed.push(entry);
      } else {
        toView.push(entry);
      }
    });
    this.toViewPRs = toView;
    this.viewedPRs = viewed;
  }
}
