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

import { Component, NgZone, OnDestroy, OnInit } from "@angular/core";
import { Subscription } from "rxjs";
import {
  PREntry,
  PullRequestsService,
} from "src/app/shared/services/pull-requests.service";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "src/app/shared/services/tauri.service";
import { invoke } from "@tauri-apps/api";

type GithubUser = {
  id: number;
  login: string;
  name: string;
  avatar_url: string;
};

@Component({
  selector: "ghd-dashboard",
  templateUrl: "./dashboard.component.html",
  styleUrls: ["./dashboard.component.scss"],
})
export class DashboardComponent
  implements OnInit, OnDestroy, TauriEventListener
{
  public iterationN = 0;
  public prs: PREntry[] = [];
  public user?: GithubUser;

  private prSubscription?: Subscription;

  public constructor(
    private zone: NgZone,
    private tauriSvc: TauriService,
    private prSvc: PullRequestsService,
  ) {}

  public ngOnInit(): void {
    this.tauriSvc.register(TauriService.events.ITERATION, this);
    this.prSubscription = this.prSvc.getPullRequests().subscribe({
      next: (entries: PREntry[]) => {
        this.prs = entries;
      },
    });
    invoke("get_user")
      .then((res) => {
        console.log("user: ", res);
        this.zone.run(() => {
          this.user = <GithubUser>res;
        });
      })
      .catch(() => {
        console.log("user not defined");
        this.zone.run(() => {
          this.user = undefined;
        });
      });
  }

  public ngOnDestroy(): void {
    this.tauriSvc.unregister(TauriService.events.ITERATION, this);
    if (!!this.prSubscription) {
      this.prSubscription.unsubscribe();
    }
  }

  public set iteration(value: number) {
    this.iterationN = value;
  }

  public get iteration(): number {
    return this.iterationN;
  }

  public getListenerID(): string {
    return "dashboard-evlistener";
  }

  public handleEvent(event: TauriListenerEvent): void {
    const evname = event.name;

    if (evname === TauriService.events.ITERATION) {
      this.handleIteration(<number>event.payload);
    } else if (evname === TauriService.events.PULL_REQUESTS_UPDATE) {
    }
  }

  private handleIteration(n: number) {
    this.zone.run(() => {
      this.iterationN = n;
    });
  }
}
