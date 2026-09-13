---
type: Roadmap
title: 'xGauntlet Roadmap & Harness Strategy'
description: 'Oversigt over planlagte, evaluerede og fravalgte agent-harnesses samt arkitektoniske adapter-slices'
status: draft
tags:
  - roadmap
  - harnesses
  - adapters
  - architecture
---

# 🗺️ xGauntlet Roadmap

Dette dokument skitserer den fremtidige integrationsstrategi og roadmap for agent-harnesses i **xGauntlet**. Hver understøttet harness implementeres som en autonom, isoleret vertikal feature-slice jf. [spec.md](spec.md), [CONTEXT.md](CONTEXT.md) og [ADR 0004: Vertical Slice Harness Adapters](docs/adr/0004-harness-adapter-slices.md).

---

## 🎯 Planlagte Harness Adapters (Kommende)

Følgende harnesses er planlagt til understøttelse i kommende versioner:

- [ ] **[Mistral Vibe](https://marketplace.visualstudio.com/items?itemName=mistralai.mistral-vibe-code)** — se [Task 022](tasks/022-mistral-vibe-harness-adapter.md) (`Aktiv`)  
  *Platform*: VS Code Extension & CLI  
  *Beskrivelse*: Mistral AI's officielle kodningsagent og workflow med `.vibe/hooks.toml` integration.

- [ ] **[Cline](https://marketplace.visualstudio.com/items?itemName=saoudrizwan.claude-dev)**  
  *Platform*: VS Code Extension  
  *Beskrivelse*: Autonom kodningsassistent i VS Code med tool-kald og CLI-eksekvering.

---

## 💡 AI-forslag til Harness-kandidater

Kandidater identificeret på baggrund af eksisterende arkitekturreferencer i repositoriet (ADR'er og opgaver), som kan evalueres til fremtidige feature-slices:

- [ ] **[OpenHands](https://github.com/All-Hands-AI/OpenHands)** (DeepSeek / OpenDevin)  
  *Reference*: Eksplicit nævnt som fremtidigt designmål i [ADR 0004](docs/adr/0004-harness-adapter-slices.md), men mangler endnu en formel adapter-specifikation og hook-mapping.

- [ ] **[Gemini CLI](https://github.com/google-gemini)**  
  *Reference*: Allerede defineret som målplatform for global plugin-distribution (`~/.gemini/config/plugins/xgauntlet`) i [Task 020](tasks/020-global-plugin-distribution-and-jit-skill-injection.md); kan formaliseres med dedikeret CLI-harness adapter.

- [ ] **[Zed](https://zed.dev/)**  
  *Reference*: Højtydende editor skrevet i Rust med egne agentic slash-commands/hooks; ideelt match til xGauntlets native Rust-motor og Zero-Daemon filosofi.

- [ ] **[Windsurf](https://codeium.com/windsurf)** (Codeium)  
  *Platform*: VS Code-baseret agentic IDE; deler adapter-karakteristika og hook-infrastruktur med Mistral Vibe og Cline.

- [ ] **[GitHub Copilot](https://github.com/features/copilot)**  
  *Platform*: VS Code Extension & CLI; udbredt økosystem, men kræver afklaring af hook-interception og integrationsmuligheder.

---

## ⛔ Bevidste Fravalg (Non-Goals)

Følgende værktøjer og platforme er bevidst fravalgt som officielle integrationer:

| Harness | Årsag / Begrundelse | Status |
| :--- | :--- | :--- |
| **Cursor** | Opkøb / tilknytning til Elon Musks X / xAI. xGauntlet prioriterer åbne, gennemsigtige standarder og leverandøruafhængige harnesses. | ❌ Afvist |