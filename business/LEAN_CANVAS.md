# NoteNest Lean Canvas

**Tagline:** Protect sensitive clinical notes and generate patient + clinician summaries, offline.

Links: GitHub Repo: https://github.com/Princey9/NoteNest | Live Site: https://princey9.github.io/NoteNest/ | Demo: https://princey9.github.io/NoteNest/demo/

---

## How to view diagrams
- GitHub renders Mermaid automatically in Markdown.
- In VS Code: open the Markdown Preview (View → Command Palette → “Markdown: Open Preview”).
- If Mermaid doesn’t render in VS Code, install the extension “Markdown Preview Mermaid Support”.

## Diagram: Lean Canvas map
```mermaid
flowchart TB
  %% Row 1
  P[Problem] --- S[Solution] --- KM[Key Metrics]
  %% Row 2
  UVP[Unique Value Proposition] --- UA[Unfair Advantage] --- CH[Channels]
  %% Row 3
  C[Customer Segments] --- CS[Cost Structure] --- R[Revenue Streams]

  P1[Messy notes slow handoffs] --> P
  P2[Sharing risks privacy exposure] --> P
  P3[Manual summarization is slow] --> P

  UVP1[Offline-first] --> UVP
  UVP2[Patient + clinician views] --> UVP
  UVP3[Deterministic outputs] --> UVP

  S1[Protected placeholders] --> S
  S2[Structured extraction] --> S
  S3[Cloak Mode scan/protect/report] --> S

  C1[Clinicians & students] --> C
  C2[Small clinics] --> C
  C3[Privacy-conscious teams] --> C

  CH1[GitHub + Releases] --> CH
  CH2[Pages demo] --> CH
  CH3[Coursework/referrals] --> CH

  R1[Free demo + CLI] --> R
  R2[Pro license] --> R
  R3[Services] --> R

  KM1[Notes processed] --> KM
  KM2[Demo → download] --> KM
  KM3[Time-to-summary] --> KM

  UA1[Offline-first + auditability] --> UA
  UA2[Dual-view outputs] --> UA

  CS1[Development] --> CS
  CS2[Docs/support] --> CS
  CS3[CI/builds] --> CS

  classDef focus fill:#0ea5e9,stroke:#0f172a,stroke-width:2px,color:#0f172a;
  classDef neutral fill:#e2e8f0,stroke:#475569,stroke-width:1px,color:#0f172a;
  class UVP,S,C,R focus;
  class P,C,CH,R,KM,UA,CS neutral;
```

## Problem (Top 3)
1) Messy clinical notes are hard to understand, slowing care and handoffs.
2) Sharing notes risks exposing sensitive data and violates privacy rules.
3) Manual summarization and de-identification are time-consuming and error-prone.

## Customer Segments
- **Primary:** Students and clinicians preparing notes for handoffs, referrals, or second opinions.
- **Secondary:** Clinics and small practices needing fast, offline prep of notes.
- **Early adopters:** Health tech students, clinical researchers, and privacy-conscious providers.

## Unique Value Proposition
**Offline-first clinical note prep** that protects sensitive fields with placeholders and produces structured patient and clinician views in seconds.

## Solution
- Protect sensitive details using deterministic placeholder rules.
- Extract clinical fields and render patient-friendly + clinician-structured views.
- Cloak Mode for scan/protect/report to enable safe sharing workflows.

## Channels
- GitHub repo + Releases
- GitHub Pages demo site
- Coursework submissions and referrals

## Revenue Streams
- Free: demo + local CLI
- Pro: individual license (offline workflows + templates)
- Services: clinic onboarding, customization, and training

## Cost Structure
- Development time
- Documentation and support
- Distribution (CI/builds)

## Key Metrics
- Notes processed per user
- Conversion rate from demo to CLI downloads
- Reduced time-to-summary (baseline vs NoteNest)
- Protected field detection counts per note

## Unfair Advantage
- Offline-first design with deterministic, auditable placeholders
- Dual-view output (patient + clinician) in one run
- Static demo that mirrors core logic for trust and transparency

---

## Diagram: Customer journey / product flow
```mermaid
flowchart LR
  A[Input messy note] --> B[Choose mode]
  B --> S1[Summarize & structure]
  B --> C1[Cloak Mode]
  S1 --> S2[Protect placeholders]
  S2 --> S3[Extract fields]
  S3 --> S4[Render Patient + Clinician View (SOAP/5C’s)]
  S4 --> Z[Export/share (offline)]

  C1 --> C2[Protect placeholders]
  C2 --> C3[Cloak scan summary]
  C3 --> C4[Cloak report (JSON/CSV)]
  C4 --> Z

  N[Offline-only. No network calls.]:::note
  Z --- N

  classDef focus fill:#22d3ee,stroke:#0f172a,stroke-width:2px,color:#0f172a;
  class S1,S2,C1,C2 focus;
  classDef note fill:#f1f5f9,stroke:#64748b,stroke-width:1px,color:#0f172a;
```
