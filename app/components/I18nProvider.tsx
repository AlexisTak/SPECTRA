'use client'

import { useEffect, useState } from 'react'
import { I18nextProvider } from 'react-i18next'
import i18n, { getStoredLanguage } from '../i18n'

export function I18nProvider({ children }: { children: React.ReactNode }) {
  const [language, setLanguage] = useState(i18n.language)

  // Applique la langue persistée après hydratation (le rendu initial part
  // toujours de la langue par défaut), puis maintient `<html lang>` à jour :
  // un lecteur d'écran doit prononcer le texte dans la bonne langue.
  useEffect(() => {
    const stored = getStoredLanguage()
    if (stored !== i18n.language) {
      void i18n.changeLanguage(stored)
    }
    const onChange = (lng: string) => setLanguage(lng)
    i18n.on('languageChanged', onChange)
    return () => {
      i18n.off('languageChanged', onChange)
    }
  }, [])

  useEffect(() => {
    document.documentElement.lang = language
  }, [language])

  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
}
