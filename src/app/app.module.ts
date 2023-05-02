import { NgModule } from "@angular/core";
import { BrowserModule } from "@angular/platform-browser";

import { AppRoutingModule } from "./app-routing.module";
import { AppComponent } from "./app.component";
import { MainLayoutComponent } from "./layout/main-layout/main-layout.component";
import { SettingsComponent } from "./pages/settings/settings.component";
import { DashboardComponent } from "./pages/dashboard/dashboard.component";
import { ReactiveFormsModule } from "@angular/forms";
import {
  NgbCollapseModule,
  NgbModalModule,
  NgbTooltipModule,
} from "@ng-bootstrap/ng-bootstrap";
import { TrackUserModalComponent } from "./pages/dashboard/track-user-modal/track-user-modal.component";
import { DashboardViewComponent } from "./pages/dashboard/dashboard-view/dashboard-view.component";
import { PullRequestsWidgetComponent } from "./pages/dashboard/pull-requests-widget/pull-requests-widget.component";
import { RepoNameComponent } from "./shared/components/repo-name/repo-name.component";
import { PRStateComponent } from "./shared/components/pr-state/pr-state.component";
import { PullRequestsTableComponent } from "./pages/dashboard/pull-requests-table/pull-requests-table.component";

@NgModule({
  declarations: [
    AppComponent,
    MainLayoutComponent,
    SettingsComponent,
    DashboardComponent,
    TrackUserModalComponent,
    DashboardViewComponent,
    PullRequestsWidgetComponent,
    RepoNameComponent,
    PRStateComponent,
    PullRequestsTableComponent,
  ],
  imports: [
    BrowserModule,
    AppRoutingModule,
    ReactiveFormsModule,
    NgbModalModule,
    NgbTooltipModule,
    NgbCollapseModule,
  ],
  providers: [],
  bootstrap: [AppComponent],
})
export class AppModule {}
