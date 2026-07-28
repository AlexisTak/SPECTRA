'use client'

/**
 * Création d'un dossier.
 *
 * La référence (`ENQ-AAAA-NNNN`) est attribuée par le backend, jamais saisie :
 * elle doit rester unique et traçable.
 */

import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { createCase, type CasePriority } from '@/lib/api'
import { PageHeader } from '@/components/layout/shell'
import {
  Button,
  ErrorBox,
  Field,
  Input,
  Panel,
  Select,
  Textarea,
} from '@/components/ui/primitives'

export default function NewCasePage() {
  const router = useRouter()
  const [titre, setTitre] = useState('')
  const [description, setDescription] = useState('')
  const [categorie, setCategorie] = useState('')
  const [priorite, setPriorite] = useState<CasePriority>('normale')
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const submit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (titre.trim() === '') return

    setSubmitting(true)
    setError(null)
    try {
      const created = await createCase({
        titre: titre.trim(),
        description: description.trim() || null,
        categorie: categorie.trim() || null,
        priorite,
        statut: 'ouvert',
      })
      router.push(`/cases/detail?id=${created.id}`)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      setSubmitting(false)
    }
  }

  return (
    <>
      <PageHeader
        title="Nouveau dossier"
        subtitle="La référence est attribuée automatiquement"
      />

      <div className="max-w-2xl p-8">
        <Panel>
          <form onSubmit={submit} className="flex flex-col gap-5">
            {error && <ErrorBox message={error} />}

            <Field label="Titre">
              <Input
                value={titre}
                onChange={(e) => setTitre(e.target.value)}
                placeholder="Objet de l'enquête"
                required
                autoFocus
              />
            </Field>

            <Field label="Description">
              <Textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                rows={4}
                placeholder="Contexte, éléments déclencheurs…"
              />
            </Field>

            <div className="grid grid-cols-2 gap-4">
              <Field label="Catégorie">
                <Input
                  value={categorie}
                  onChange={(e) => setCategorie(e.target.value)}
                  placeholder="Optionnel"
                />
              </Field>

              <Field label="Priorité">
                <Select
                  value={priorite}
                  onChange={(e) => setPriorite(e.target.value as CasePriority)}
                >
                  <option value="basse">Basse</option>
                  <option value="normale">Normale</option>
                  <option value="haute">Haute</option>
                  <option value="urgente">Urgente</option>
                </Select>
              </Field>
            </div>

            <div className="flex gap-3 pt-1">
              <Button
                type="submit"
                variant="primary"
                disabled={submitting || titre.trim() === ''}
              >
                {submitting ? 'Création…' : 'Créer le dossier'}
              </Button>
              <Button type="button" onClick={() => router.back()}>
                Annuler
              </Button>
            </div>
          </form>
        </Panel>
      </div>
    </>
  )
}
