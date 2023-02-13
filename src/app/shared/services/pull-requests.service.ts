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

import { Injectable } from "@angular/core";
import { BehaviorSubject } from "rxjs";
import {
  TauriEventListener,
  TauriListenerEvent,
  TauriService,
} from "./tauri.service";

export type PREntry = {
  id: number;
  title: string;
  age_str: string;
};

type PRList = {
  entries: PREntry[];
};

@Injectable({
  providedIn: "root",
})
export class PullRequestsService implements TauriEventListener {
  private pr_subject: BehaviorSubject<PREntry[]> = new BehaviorSubject<
    PREntry[]
  >([]);

  public constructor(private tauriSvc: TauriService) {
    this.tauriSvc.register(TauriService.events.PULL_REQUESTS_UPDATE, this);
  }

  public getListenerID(): string {
    return "pr-svc-listener";
  }

  public handleEvent(event: TauriListenerEvent): void {
    if (event.name !== TauriService.events.PULL_REQUESTS_UPDATE) {
      console.error("pr-svc received event not pull_request_update!");
      return;
    }

    const list = <PRList>event.payload;
    this.pr_subject.next(list.entries);
  }

  public getPullRequests(): BehaviorSubject<PREntry[]> {
    return this.pr_subject;
  }
}
