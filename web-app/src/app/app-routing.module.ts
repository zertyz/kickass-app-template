import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { HomePage } from "./home-page/home-page.component";
import { RestServiceExplorerPage } from "./rest-service-explorer-page/rest-service-explorer-page.component";
import { GetServiceExplorerPage } from "./get-service-explorer-page/get-service-explorer-page.component";
import { PostServiceExplorerPage } from "./post-service-explorer-page/post-service-explorer-page.component";

const routes: Routes = [
  { path: '', redirectTo: '/home', pathMatch: 'full' },
  { path: 'home', component: HomePage },
  { path: 'rest-service-explorer', component: RestServiceExplorerPage },
  { path: 'get-service-explorer', component: GetServiceExplorerPage },
  { path: 'post-service-explorer', component: PostServiceExplorerPage },
];

@NgModule({
  imports: [RouterModule.forRoot(routes, {
    initialNavigation: 'enabled'
})],
  exports: [RouterModule]
})
export class AppRoutingModule { }
