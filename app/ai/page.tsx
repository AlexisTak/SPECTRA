"use client";

import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppShell, PageHeader } from "@/components/layout/shell";
import {
  Panel,
  Button,
  Input,
  Textarea,
  Select,
  ErrorBox,
} from "@/components/ui/primitives";
import { Loader2, Brain, AlertTriangle, CheckCircle, XCircle } from "lucide-react";

/* ------------------------------------------------------------------ */
// Types
/* ------------------------------------------------------------------ */

interface AiStatus {
  available: boolean;
  backendName: string;
  model: string | null;
}

interface AiSuggestOutput {
  description: string;
  confidence: number;
}

interface AiDuplicateResult {
  idA: string;
  idB: string;
  similarity: number;
}

interface AiRagSource {
  sourceType: string;
  sourceId: string;
  text: string;
  score: number;
}

interface AiRagOutput {
  answer: string;
  sources: AiRagSource[];
}

/* ------------------------------------------------------------------ */
// Composant principal
/* ------------------------------------------------------------------ */

export default function AiPage() {
  const [status, setStatus] = useState<AiStatus | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /* ---- Onglets ---- */
  const [tab, setTab] = useState<
    "status" | "summarize" | "ner" | "pivots" | "duplicates" | "rag"
  >("status");

  /* ---- Summarize ---- */
  const [sumText, setSumText] = useState("");
  const [sumResult, setSumResult] = useState("");

  /* ---- NER ---- */
  const [nerText, setNerText] = useState("");
  const [nerCaseId, setNerCaseId] = useState("");
  const [nerResult, setNerResult] = useState<any[]>([]);

  /* ---- Pivots ---- */
  const [pivEntityId, setPivEntityId] = useState("");
  const [pivKind, setPivKind] = useState("username");
  const [pivLabel, setPivLabel] = useState("");
  const [pivValue, setPivValue] = useState("");
  const [pivCtx, setPivCtx] = useState("");
  const [pivResult, setPivResult] = useState<AiSuggestOutput[]>([]);

  /* ---- Duplicates ---- */
  const [dupEntities, setDupEntities] = useState<string>("");
  const [dupThresh, setDupThresh] = useState(0.7);
  const [dupResult, setDupResult] = useState<AiDuplicateResult[]>([]);

  /* ---- RAG ---- */
  const [ragQuestion, setRagQuestion] = useState("");
  const [ragChunks, setRagChunks] = useState<string>("");
  const [ragResult, setRagResult] = useState<AiRagOutput | null>(null);

  /* ---------------------------------------------------------------- */

  useEffect(() => {
    fetchStatus();
  }, []);

  async function fetchStatus() {
    try {
      const s: AiStatus = await invoke("ai_status");
      setStatus(s);
      setError(null);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  }

  function guardAvailable(): boolean {
    if (!status?.available) {
      setError("IA indisponible — installez Ollama et lancez-le.");
      return false;
    }
    return true;
  }

  /* ---- Actions ---- */

  async function handleSummarize() {
    if (!guardAvailable()) return;
    setLoading(true);
    setError(null);
    try {
      const res: string = await invoke("ai_summarize", {
        input: { text: sumText },
      });
      setSumResult(res);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleNer() {
    if (!guardAvailable()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await invoke("ai_extract_entities", {
        input: { text: nerText, caseId: nerCaseId },
      });
      setNerResult(res as any[]);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handlePivots() {
    if (!guardAvailable()) return;
    setLoading(true);
    setError(null);
    try {
      const res: AiSuggestOutput[] = await invoke("ai_suggest_pivots", {
        input: {
          entityId: pivEntityId,
          entityKind: pivKind,
          displayLabel: pivLabel,
          canonicalValue: pivValue,
          context: pivCtx,
        },
      });
      setPivResult(res);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleDuplicates() {
    if (!guardAvailable()) return;
    setLoading(true);
    setError(null);
    try {
      const lines = dupEntities.trim().split("\n").filter(Boolean);
      const entities = lines.map((l) => {
        const [id, kind, label, val] = l.split("|");
        return { id, kind, displayLabel: label, canonicalValue: val };
      });
      const res: AiDuplicateResult[] = await invoke("ai_detect_duplicates", {
        input: { entities, threshold: dupThresh },
      });
      setDupResult(res);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleRag() {
    if (!guardAvailable()) return;
    setLoading(true);
    setError(null);
    try {
      const lines = ragChunks.trim().split("\n---\n").filter(Boolean);
      const chunks = lines.map((text, i) => ({
        id: `chunk-${i}`,
        sourceType: "note",
        sourceId: `note-${i}`,
        text,
      }));
      const res: AiRagOutput = await invoke("ai_rag_query", {
        input: { question: ragQuestion, chunks },
      });
      setRagResult(res);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }

  /* ---------------------------------------------------------------- */

  const tabs = [
    { key: "status" as const, label: "Statut" },
    { key: "summarize" as const, label: "Résumé" },
    { key: "ner" as const, label: "NER" },
    { key: "pivots" as const, label: "Pivots" },
    { key: "duplicates" as const, label: "Doublons" },
    { key: "rag" as const, label: "RAG" },
  ];

  return (
    <AppShell>
      <div className="space-y-6 px-8 py-5">
        {/* En-tête */}
        <PageHeader
          title="IA locale"
          subtitle="Outils d'analyse augmentée — 100 % offline, marqués INFERRED"
          action={
            status && (
              <div className="flex items-center gap-2">
                {status.available ? (
                  <span className="inline-flex items-center gap-1.5 rounded border border-emerald-500/40 bg-emerald-500/10 px-2 py-1 text-xs font-medium text-emerald-300">
                    <CheckCircle className="h-3 w-3" />
                    {status.backendName}
                    {status.model ? ` — ${status.model}` : ""}
                  </span>
                ) : (
                  <span className="inline-flex items-center gap-1.5 rounded border border-rose-500/40 bg-rose-500/10 px-2 py-1 text-xs font-medium text-rose-300">
                    <XCircle className="h-3 w-3" />
                    IA hors ligne
                  </span>
                )}
              </div>
            )
          }
        />

        {/* Navigation par onglets */}
        <div className="flex flex-wrap gap-2">
          {tabs.map((t) => (
            <Button
              key={t.key}
              variant={tab === t.key ? "primary" : "default"}
              onClick={() => setTab(t.key)}
            >
              {t.label}
            </Button>
          ))}
        </div>

        {error && <ErrorBox message={error} />}

        {/* ======================== ONGLET STATUT ======================== */}
        {tab === "status" && (
          <Panel title="Backend IA">
            <div className="space-y-4">
              {status ? (
                <div className="space-y-2 text-sm">
                  <p>
                    <span className="font-medium">Disponible :</span>{" "}
                    {status.available ? "Oui" : "Non"}
                  </p>
                  <p>
                    <span className="font-medium">Backend :</span>{" "}
                    {status.backendName}
                  </p>
                  <p>
                    <span className="font-medium">Modèle :</span>{" "}
                    {status.model ?? "—"}
                  </p>
                </div>
              ) : (
                <div className="flex items-center gap-2 text-sm text-[var(--color-muted)]">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Détection en cours…
                </div>
              )}

              <hr className="border-[var(--color-edge)]" />

              <div className="space-y-2 text-xs text-[var(--color-muted)]">
                <p className="flex items-center gap-2">
                  <AlertTriangle className="h-3 w-3 text-amber-400" />
                  Toute sortie IA est marquée <strong className="text-amber-300">INFERRED</strong> et
                  exclue des rapports par défaut.
                </p>
                <p>
                  Les résultats nécessitent une validation humaine avant
                  intégration au graphe ou au dossier.
                </p>
              </div>
            </div>
          </Panel>
        )}

        {/* ======================== ONGLET RÉSUMÉ ======================== */}
        {tab === "summarize" && (
          <Panel title="Résumé automatique">
            <div className="space-y-4">
              <Textarea
                placeholder="Collez le texte à résumer…"
                value={sumText}
                onChange={(e) => setSumText(e.target.value)}
                rows={6}
              />
              <Button
                variant="primary"
                onClick={handleSummarize}
                disabled={loading || !sumText}
              >
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Résumer
              </Button>
              {sumResult && (
                <div className="rounded border border-dashed border-amber-500/40 bg-amber-500/5 p-4">
                  <span className="mb-2 inline-block rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-bold text-amber-300">
                    INFERRED
                  </span>
                  <p className="whitespace-pre-wrap text-sm">{sumResult}</p>
                </div>
              )}
            </div>
          </Panel>
        )}

        {/* ======================== ONGLET NER ======================== */}
        {tab === "ner" && (
          <Panel title="Extraction d'entités (NER)">
            <div className="space-y-4">
              <Input
                placeholder="ID du dossier (caseId)"
                value={nerCaseId}
                onChange={(e) => setNerCaseId(e.target.value)}
              />
              <Textarea
                placeholder="Texte libre à analyser…"
                value={nerText}
                onChange={(e) => setNerText(e.target.value)}
                rows={6}
              />
              <Button
                variant="primary"
                onClick={handleNer}
                disabled={loading || !nerText}
              >
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Extraire
              </Button>
              {nerResult.length > 0 && (
                <div className="space-y-2">
                  {nerResult.map((o: any, i: number) => (
                    <div
                      key={i}
                      className="rounded border border-dashed border-amber-500/40 bg-amber-500/5 p-3 text-sm"
                    >
                      <span className="mb-1 inline-block rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-bold text-amber-300">
                        INFERRED
                      </span>
                      <p>
                        <strong>{o.predicate}</strong> → {o.value}
                      </p>
                      <p className="text-xs text-[var(--color-muted)]">
                        Confiance {o.confidence} · Source {o.provenance}
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </Panel>
        )}

        {/* ======================== ONGLET PIVOTS ======================== */}
        {tab === "pivots" && (
          <Panel title="Suggestion de pivots">
            <div className="space-y-4">
              <div className="grid grid-cols-1 gap-3 md:grid-cols-2">
                <Input
                  placeholder="ID entité"
                  value={pivEntityId}
                  onChange={(e) => setPivEntityId(e.target.value)}
                />
                <Select
                  value={pivKind}
                  onChange={(e) => setPivKind(e.target.value)}
                >
                  <option value="username">Username</option>
                  <option value="person">Person</option>
                  <option value="organization">Organization</option>
                  <option value="location">Location</option>
                  <option value="emailaddress">EmailAddress</option>
                  <option value="phonenumber">PhoneNumber</option>
                  <option value="domain">Domain</option>
                </Select>
                <Input
                  placeholder="Label affiché"
                  value={pivLabel}
                  onChange={(e) => setPivLabel(e.target.value)}
                />
                <Input
                  placeholder="Valeur canonique"
                  value={pivValue}
                  onChange={(e) => setPivValue(e.target.value)}
                />
              </div>
              <Textarea
                placeholder="Contexte optionnel…"
                value={pivCtx}
                onChange={(e) => setPivCtx(e.target.value)}
                rows={3}
              />
              <Button
                variant="primary"
                onClick={handlePivots}
                disabled={loading || !pivEntityId || !pivValue}
              >
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Suggérer
              </Button>
              {pivResult.length > 0 && (
                <div className="space-y-2">
                  {pivResult.map((s, i) => (
                    <div
                      key={i}
                      className="rounded border border-dashed border-amber-500/40 bg-amber-500/5 p-3 text-sm"
                    >
                      <span className="mb-1 inline-block rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-bold text-amber-300">
                        INFERRED
                      </span>
                      <p>{s.description}</p>
                      <p className="text-xs text-[var(--color-muted)]">
                        Confiance {(s.confidence * 100).toFixed(0)}%
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </Panel>
        )}

        {/* ======================== ONGLET DUPLICATES ======================== */}
        {tab === "duplicates" && (
          <Panel title="Détection de doublons">
            <div className="space-y-4">
              <Textarea
                placeholder={`Format : id|kind|label|value\nEx : e1|person|Jean Dupont|jean.dupont\ne2|person|J. Dupont|jean.dupont`}
                value={dupEntities}
                onChange={(e) => setDupEntities(e.target.value)}
                rows={6}
              />
              <div className="flex items-center gap-3">
                <label className="text-sm font-medium">Seuil :</label>
                <Input
                  type="number"
                  step={0.05}
                  min={0}
                  max={1}
                  value={String(dupThresh)}
                  onChange={(e) => setDupThresh(parseFloat(e.target.value))}
                  className="w-24"
                />
              </div>
              <Button
                variant="primary"
                onClick={handleDuplicates}
                disabled={loading || !dupEntities}
              >
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Détecter
              </Button>
              {dupResult.length > 0 && (
                <div className="space-y-2">
                  {dupResult.map((d, i) => (
                    <div
                      key={i}
                      className="rounded border border-dashed border-amber-500/40 bg-amber-500/5 p-3 text-sm"
                    >
                      <span className="mb-1 inline-block rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-bold text-amber-300">
                        INFERRED
                      </span>
                      <p>
                        <strong>{d.idA}</strong> ↔{" "}
                        <strong>{d.idB}</strong>
                      </p>
                      <p className="text-xs text-[var(--color-muted)]">
                        Similarité {(d.similarity * 100).toFixed(0)}%
                      </p>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </Panel>
        )}

        {/* ======================== ONGLET RAG ======================== */}
        {tab === "rag" && (
          <Panel title="Recherche augmentée (RAG)">
            <div className="space-y-4">
              <Input
                placeholder="Question…"
                value={ragQuestion}
                onChange={(e) => setRagQuestion(e.target.value)}
              />
              <Textarea
                placeholder={`Documents (séparés par \\n---\\n)…`}
                value={ragChunks}
                onChange={(e) => setRagChunks(e.target.value)}
                rows={8}
              />
              <Button
                variant="primary"
                onClick={handleRag}
                disabled={loading || !ragQuestion || !ragChunks}
              >
                {loading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Interroger
              </Button>
              {ragResult && (
                <div className="space-y-4">
                  <div className="rounded border border-dashed border-amber-500/40 bg-amber-500/5 p-4">
                    <span className="mb-2 inline-block rounded border border-amber-500/40 bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-bold text-amber-300">
                      INFERRED
                    </span>
                    <p className="whitespace-pre-wrap text-sm">{ragResult.answer}</p>
                  </div>
                  {ragResult.sources.length > 0 && (
                    <div className="space-y-2">
                      <p className="text-sm font-medium">Sources utilisées :</p>
                      {ragResult.sources.map((s, i) => (
                        <div
                          key={i}
                          className="rounded border border-[var(--color-edge)] bg-[var(--color-surface)] p-2 text-xs"
                        >
                          <p className="font-medium">
                            {s.sourceType} — {s.sourceId}
                          </p>
                          <p className="line-clamp-3 text-[var(--color-muted)]">
                            {s.text}
                          </p>
                          <p className="mt-1 text-[var(--color-muted)]">
                            Score {(s.score * 100).toFixed(1)}%
                          </p>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>
          </Panel>
        )}
      </div>
    </AppShell>
  );
}
