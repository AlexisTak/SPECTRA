import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import fr from './locales/fr.json'
import en from './locales/en.json'

const resources = {
  fr: { translation: fr },
  en: { translation: en },
}

export const SUPPORTED_LANGUAGES = ['fr', 'en'] as const
export const DEFAULT_LANGUAGE = 'fr'
const STORAGE_KEY = 'spectra.language'

/**
 * Langue choisie par l'analyste. Stockage strictement local (localStorage) :
 * aucune détection réseau, aucune requête sortante.
 */
export function getStoredLanguage(): string {
  if (typeof window === 'undefined') return DEFAULT_LANGUAGE
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY)
    return stored && (SUPPORTED_LANGUAGES as readonly string[]).includes(stored)
      ? stored
      : DEFAULT_LANGUAGE
  } catch {
    // localStorage indisponible (WebView verrouillée) : repli silencieux.
    return DEFAULT_LANGUAGE
  }
}

export function setStoredLanguage(lng: string): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.setItem(STORAGE_KEY, lng)
  } catch {
    // Persistance impossible : le choix reste valable pour la session.
  }
}

i18n
  .use(initReactI18next)
  .init({
    resources,
    // Le rendu initial part toujours de la langue par défaut ; la langue
    // stockée est appliquée côté client par `I18nProvider`, ce qui évite
    // toute divergence d'hydratation.
    lng: DEFAULT_LANGUAGE,
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false,
    },
  })

export default i18n
