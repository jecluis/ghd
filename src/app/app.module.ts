import { NgModule } from "@angular/core";
import { BrowserModule } from "@angular/platform-browser";

import { AppRoutingModule } from "./app-routing.module";
import { AppComponent } from "./app.component";
import { MainLayoutComponent } from "./layout/main-layout/main-layout.component";
import { SettingsComponent } from "./pages/settings/settings.component";
import { DashboardComponent } from "./pages/dashboard/dashboard.component";
import { ReactiveFormsModule } from "@angular/forms";

@NgModule({
  declarations: [
    AppComponent,
    MainLayoutComponent,
    SettingsComponent,
    DashboardComponent,
  ],
  imports: [BrowserModule, AppRoutingModule, ReactiveFormsModule],
  providers: [],
  bootstrap: [AppComponent],
})
export class AppModule {}
