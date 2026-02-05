# NoteNest Diagrams (Mermaid)

## How to view diagrams
- GitHub renders Mermaid automatically in Markdown.
- For local previews in VS Code, install a Mermaid extension (e.g., “Markdown Preview Mermaid Support”).
- Alternatively, use GitHub.dev (press `.` in the repo) or paste the diagram code into https://mermaid.live.

Verification checklist:
- Open README on GitHub and confirm diagrams render.
- Open VS Code preview and confirm diagrams render (with extension installed).
- If not, paste Mermaid code into mermaid.live.

---

## Offline pipeline (README)
```mermaid
flowchart LR
  A[Raw clinical note] --> B[Normalize & clean]
  B --> C[Protect sensitive fields]
  C --> D[Extract clinical fields]
  D --> E[Fill typed templates]
  E --> F[Text outputs]
  E --> G[JSON outputs]
  E --> H[Coverage report]
```

## Architecture overview (README)
```mermaid
flowchart TB
  CLI[CLI: notenest] --> Pipeline[Core pipeline]
  Pipeline --> Protect[Protect module]
  Pipeline --> Extract[Extract/Summary module]
  Pipeline --> Render[Render module]
  Render --> TextOut[Text output]
  Render --> JsonOut[JSON output]
  Web[docs/ demo JS] --> Mirror[Client-side mirror of pipeline]
  Mirror --> DemoOut[Demo outputs]
```

---

## Lean Canvas map (business/LEAN_CANVAS.md)
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
  KM2[Demo -> download] --> KM
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

## Customer journey / product flow (business/LEAN_CANVAS.md)
```mermaid
flowchart LR
  A[Input messy note] --> B[Choose mode]
  B --> C[Summarize and structure]
  B --> D[Cloak Mode]
  C --> E[Protect placeholders]
  E --> F[Extract fields]
  F --> G[Render Patient and Clinician View with SOAP and 5Cs]
  G --> H[Export and share offline]
  D --> I[Protect placeholders]
  I --> J[Cloak scan summary]
  J --> K[Cloak report in JSON and CSV]
  K --> H
  H --> N[Offline only, no network calls]

  classDef note fill:#f1f5f9,stroke:#64748b,stroke-width:1px,color:#0f172a;
  class N note;
```

Troubleshooting Mermaid: GitHub renders Mermaid automatically. For local previews in VS Code, install a Mermaid preview extension.
