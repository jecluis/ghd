import { TestBed } from "@angular/core/testing";

import { UserPullRequestsService } from "./user-pull-requests.service";

describe("UserPullRequestsService", () => {
  let service: UserPullRequestsService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UserPullRequestsService);
  });

  it("should be created", () => {
    expect(service).toBeTruthy();
  });
});
