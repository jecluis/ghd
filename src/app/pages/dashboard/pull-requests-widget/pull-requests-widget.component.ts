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
import formatDistance from "date-fns/formatDistance";
import toDate from "date-fns/toDate";
import { interval, map, Observable } from "rxjs";

type PRTableEntry = {
  id: number;
  number: number;
  title: string;
  author: string;
  repoOwner: string;
  url: string;
  repoName: string;
  state: string;
  lastUpdate: number;
  lastUpdateObs: Observable<string>;
  reviewDecision: string;
};

type TrackedPRs = {
  toView: PRTableEntry[];
  viewed: PRTableEntry[];
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

  public ownPRs: TrackedPRs = {
    toView: [],
    viewed: [],
  };
  public involved: TrackedPRs = {
    toView: [],
    viewed: [],
  };

  public isMarkingViewed = false;
  public markingViewed?: number;

  public constructor(private zone: NgZone, private tauriSvc: TauriService) {}

  public ngOnInit(): void {
    this.tauriSvc.register(TauriService.events.USER_DATA_UPDATE, this);
    this.updateUser().then(() => {});
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
        this.updateUser().then(() => {});
      });
    }
  }

  public markViewed(pr: PRTableEntry): void {
    this.isMarkingViewed = true;
    this.markingViewed = pr.id;
    this.tauriSvc
      .markPullRequestViewed(pr.id)
      .then(() => {
        this.updateUser().then(() => {
          this.isMarkingViewed = false;
          this.markingViewed = undefined;
        });
      })
      .catch(() => {
        console.error(`unable to mark PR id ${pr.id} as viewed`);
      });
  }

  public getDateDiff(value: number): string {
    // we get a timestamp in seconds, but we need it in milliseconds.
    let now = new Date();
    let updatedAt = toDate(value * 1000);
    return formatDistance(updatedAt, now);
  }

  private async updateUser(): Promise<void> {
    try {
      let prs = await this.tauriSvc.getPullRequestsByAuthor(this.user.login);
      this.ownPRs = this.processPRs(prs);

      let involved = await this.tauriSvc.getInvolvedPullRequests(
        this.user.login,
      );
      this.involved = this.processPRs(involved);
    } catch (err) {
      console.error("unable to update user: ", err);
    }
  }

  private processPRs(prs: PullRequestEntry[]): TrackedPRs {
    let toView: PRTableEntry[] = [];
    let viewed: PRTableEntry[] = [];
    prs.forEach((pr: PullRequestEntry) => {
      let entry: PRTableEntry = {
        id: pr.id,
        number: pr.number,
        title: pr.title,
        author: pr.author,
        repoOwner: pr.repo_owner,
        repoName: pr.repo_name,
        url: pr.url,
        state: pr.state,
        lastUpdate: pr.updated_at,
        lastUpdateObs: interval(1000).pipe(
          map(() => this.getDateDiff(pr.updated_at)),
        ),
        reviewDecision: pr.review_decision,
      };
      if (!!pr.last_viewed && pr.last_viewed >= pr.updated_at) {
        viewed.push(entry);
      } else {
        toView.push(entry);
      }
    });
    return { toView: toView, viewed: viewed };
  }
}
