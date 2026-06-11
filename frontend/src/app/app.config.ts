import { ApplicationConfig, APP_INITIALIZER } from '@angular/core';
import { provideRouter } from '@angular/router';
import { routes } from './app.routes';
import { AuthService } from './core/services/auth.service';
import { I18nService } from './core/services/i18n.service';

function initializeApp(auth: AuthService, i18n: I18nService) {
  return async () => {
    await auth.init();
    await i18n.load();
  };
}

export const appConfig: ApplicationConfig = {
  providers: [
    provideRouter(routes),
    {
      provide: APP_INITIALIZER,
      useFactory: initializeApp,
      deps: [AuthService, I18nService],
      multi: true,
    },
  ],
};
