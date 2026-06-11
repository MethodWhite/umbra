import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';

export type Language = 'es' | 'en' | 'zh';

@Injectable({ providedIn: 'root' })
export class I18nService {
  private translations: Record<string, any> = {};
  private langSubject = new BehaviorSubject<Language>('es');
  private loaded = false;

  currentLang$: Observable<Language> = this.langSubject.asObservable();

  get currentLang(): Language {
    return this.langSubject.value;
  }

  async load(lang?: Language): Promise<void> {
    const l = lang || this.detectLanguage();
    try {
      const data = await import(`../../../assets/i18n/${l}.json`);
      this.translations[l] = data;
      this.langSubject.next(l);
      this.loaded = true;
      document.documentElement.lang = l;
      localStorage.setItem('umbra_lang', l);
    } catch (e) {
      console.error(`[i18n] failed to load ${l}`, e);
      if (l !== 'en') {
        await this.load('en');
      }
    }
  }

  t(key: string): string {
    if (!this.loaded) return key;
    const lang = this.currentLang;
    return this.translations[lang]?.[key] || this.translations['en']?.[key] || key;
  }

  setLanguage(lang: Language): void {
    if (this.translations[lang]) {
      this.langSubject.next(lang);
      document.documentElement.lang = lang;
      localStorage.setItem('umbra_lang', lang);
    } else {
      this.load(lang);
    }
  }

  private detectLanguage(): Language {
    const saved = localStorage.getItem('umbra_lang') as Language | null;
    if (saved && ['es', 'en', 'zh'].includes(saved)) return saved;
    const browser = navigator.language.split('-')[0] as Language;
    if (['es', 'en', 'zh'].includes(browser)) return browser;
    return 'en';
  }
}
