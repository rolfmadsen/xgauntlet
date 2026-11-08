# Verification Report

**Task ID**: `075-domain-orphan-resolution-and-ast-symbol-unification`  
**Task Title**: Task 075: Domain Orphan Resolution & AST Symbol Unification  
**Verdict**: `INCOMPLETE`  
**Execution Origin**: `LOCAL`  
**Source Manifest Digest**: `38b7038ce3923bd0dd5748469a6bdad74225bf2df4d1c02b34a0652fd1e26607`  
**Timestamp**: `2026-11-08T21:21:57Z`  
**Head**: `4d30c2e`  
**Commit**: `4d30c2e`  

## Acceptance Criteria

- [ ] **AC1 (Symbol Table Shadowing Elimination):**
- [ ] AST-scanneren i `parser.rs` prioriterer produktionsfiler (`src/`) og domænemodeller (`is_domain_model = true`) frem for test-filer (`tests/`), så mock-modeller aldrig skygger for ægte typer.
- [ ] `VerificationReport *-- TaskContract : task_contract` og `DoctorCheckItem *-- DoctorCategory : category` renderes automatisk i `architecture.mermaid`.
- [ ] **AC2 (Policy Modeller & Hook/Adapter Forbindelse):**
- [ ] `features/policy/models.rs` definerer eller re-eksporterer de kanoniske policy-modeller.
- [ ] `PolicyDecision *-- DecisionVerdict : verdict` og `CapabilityRequest *-- ToolActionType : action_type` renderes.
- [ ] `AdapterHookVerdict *-- DecisionVerdict : decision` og `NormalizedToolCall *-- ToolActionType : action_type` renderes.
- [ ] **AC3 (Attestation Forældre/Børn Relationer):**
- [ ] `AttestationReport` renderes i konceptuelt diagram med relationer til `AttestationStatus` og `TrustDecision`.
- [ ] **AC4 (Analyse DTO Integration):**
- [ ] `BlastRadiusReport` og `DiscoveredHarness` er ikke længere isolerede øer i det konceptuelle klassediagram.
- [ ] **AC5 (Multi-Layer Verifikation & Invariant Kontrol):**
- [ ] `cargo test` og `xgauntlet verify --task 075 --save` passerer uden fejl eller regressionsbrud.

---

## Verification Checks

| Check Name | Status | Exit Code | Duration (s) |
|---|---|---|---|
| `architecture` | `PASSED` | `0` | `0.097s` |
| `format` | `PASSED` | `0` | `0.253s` |
| `lint` | `PASSED` | `0` | `0.171s` |
| `unit` | `PASSED` | `0` | `32.088s` |
| `npm` | `PASSED` | `0` | `0.896s` |
| `invariants` | `PASSED` | `0` | `0.082s` |
| `mutation-testing-gauntlet` | `FAILED` | `101` | `0.010s` |

---

### 📜 Multi-Session Audit Lineage

| Session | Rolle | Fase | Commit | Tidsstempel | Status |
|---|---|---|---|---|---|
| `sess-0e5ff17b` | `System Architect` | `SPEC` | `0e5ff17` | `2026-10-25T14:51:15Z` | `VALID` |
| `sess-589985b0` | `TDD Craftsman` | `RED` | `589985b` | `2026-10-25T15:00:27Z` | `VALID` |
| `sess-89220d3e` | `TDD Craftsman` | `GREEN` | `89220d3` | `2026-10-25T15:05:09Z` | `VALID` |
| `sess-0de064f4` | `TDD Craftsman` | `REFACTOR` | `0de064f` | `2026-10-25T15:06:38Z` | `VALID` |
| `sess-26bd5b99` | `Software Engineer` | `DONE` | `26bd5b9` | `2026-10-25T14:16:13Z` | `VALID` |
| `sess-4a8a2566` | `System Architect` | `SPEC` | `4a8a256` | `2026-10-25T14:43:26Z` | `VALID` |
| `sess-4a8a2566` | `TDD Craftsman` | `RED` | `c1b2b2d` | `2026-10-25T14:50:03Z` | `VALID` |
| `sess-7793d1b7` | `TDD Craftsman` | `GREEN` | `7793d1b` | `2026-10-25T15:08:20Z` | `VALID` |
| `sess-58ba84b2` | `TDD Craftsman` | `REFACTOR` | `58ba84b` | `2026-10-25T15:08:57Z` | `VALID` |
| `sess-cfddffd7` | `Software Engineer` | `DONE` | `cfddffd` | `2026-10-25T15:12:16Z` | `VALID` |
| `sess-59e83396` | `System Architect` | `SPEC` | `59e8339` | `2026-10-25T15:21:43Z` | `VALID` |
| `sess-e36bbb3c` | `TDD Craftsman` | `RED` | `e36bbb3` | `2026-10-25T15:22:47Z` | `VALID` |
| `sess-3f63030e` | `TDD Craftsman` | `GREEN` | `3f63030` | `2026-10-25T15:28:34Z` | `VALID` |
| `sess-21582552` | `TDD Craftsman` | `REFACTOR` | `2158255` | `2026-10-25T15:32:13Z` | `VALID` |
| `sess-a70236ba` | `Software Engineer` | `DONE` | `a70236b` | `2026-10-25T15:33:55Z` | `VALID` |
| `sess-f50fd4dc` | `Software Engineer` | `DONE` | `f50fd4d` | `2026-10-25T15:34:59Z` | `VALID` |
| `sess-1d7f44ec` | `TDD Craftsman` | `RED` | `1d7f44e` | `2026-10-25T16:01:02Z` | `VALID` |
| `sess-9dc6319c` | `TDD Craftsman` | `GREEN` | `9dc6319` | `2026-10-25T16:06:50Z` | `VALID` |
| `sess-2c9ed01a` | `TDD Craftsman` | `REFACTOR` | `2c9ed01` | `2026-10-25T16:07:41Z` | `VALID` |
| `sess-fc17224e` | `Software Engineer` | `DONE` | `fc17224` | `2026-10-25T16:10:58Z` | `VALID` |
| `sess-8050b79d` | `System Architect` | `SPEC` | `8050b79` | `2026-10-25T16:19:49Z` | `VALID` |
| `sess-67315fa5` | `TDD Craftsman` | `RED` | `67315fa` | `2026-10-25T16:21:56Z` | `VALID` |
| `sess-8c983637` | `TDD Craftsman` | `GREEN` | `8c98363` | `2026-10-25T16:27:56Z` | `VALID` |
| `sess-24ac25ac` | `TDD Craftsman` | `REFACTOR` | `24ac25a` | `2026-10-25T16:28:37Z` | `VALID` |
| `sess-1884ade7` | `Software Engineer` | `DONE` | `1884ade` | `2026-10-25T16:29:59Z` | `VALID` |
| `sess-05fd66fd` | `System Architect` | `SPEC` | `05fd66f` | `2026-10-25T16:41:15Z` | `VALID` |
| `sess-de771be3` | `TDD Craftsman` | `RED` | `de771be` | `2026-10-25T16:47:22Z` | `VALID` |
| `sess-0c135645` | `TDD Craftsman` | `GREEN` | `0c13564` | `2026-10-25T16:55:51Z` | `VALID` |
| `sess-fdbda2ae` | `TDD Craftsman` | `REFACTOR` | `fdbda2a` | `2026-10-25T16:59:02Z` | `VALID` |
| `sess-aa21fa7f` | `Software Engineer` | `DONE` | `aa21fa7` | `2026-10-25T17:01:29Z` | `VALID` |
| `sess-88b26411` | `TDD Craftsman` | `RED` | `88b2641` | `2026-10-25T17:19:44Z` | `VALID` |
| `sess-e40eb6c5` | `TDD Craftsman` | `GREEN` | `e40eb6c` | `2026-10-25T17:23:59Z` | `VALID` |
| `sess-2604f95e` | `TDD Craftsman` | `REFACTOR` | `2604f95` | `2026-10-25T17:24:43Z` | `VALID` |
| `sess-4ead69e2` | `Software Engineer` | `DONE` | `4ead69e` | `2026-10-25T17:26:10Z` | `VALID` |
| `sess-94e8a286` | `TDD Craftsman` | `RED` | `94e8a28` | `2026-10-25T19:19:28Z` | `VALID` |
| `sess-fc38f6bb` | `TDD Craftsman` | `GREEN` | `fc38f6b` | `2026-10-25T19:22:06Z` | `VALID` |
| `sess-499be1ee` | `TDD Craftsman` | `REFACTOR` | `499be1e` | `2026-10-25T19:22:38Z` | `VALID` |
| `sess-dc9e26a3` | `Software Engineer` | `DONE` | `dc9e26a` | `2026-10-25T19:26:06Z` | `VALID` |
| `sess-2f2e2454` | `TDD Craftsman` | `RED` | `2f2e245` | `2026-10-25T19:34:52Z` | `VALID` |
| `sess-30e4ad7d` | `TDD Craftsman` | `GREEN` | `30e4ad7` | `2026-10-25T19:41:18Z` | `VALID` |
| `sess-9e9ed74b` | `TDD Craftsman` | `REFACTOR` | `9e9ed74` | `2026-10-25T19:43:16Z` | `VALID` |
| `sess-a15bf0d4` | `Software Engineer` | `DONE` | `a15bf0d` | `2026-10-25T19:48:47Z` | `VALID` |
| `sess-df1eb13b` | `System Architect` | `SPEC` | `df1eb13` | `2026-10-25T20:20:29Z` | `VALID` |
| `sess-76ff794c` | `TDD Craftsman` | `RED` | `76ff794` | `2026-10-25T20:21:37Z` | `VALID` |
| `sess-009bffa7` | `TDD Craftsman` | `GREEN` | `009bffa` | `2026-10-25T20:25:55Z` | `VALID` |
| `sess-94ccfcec` | `TDD Craftsman` | `REFACTOR` | `94ccfce` | `2026-10-25T20:27:03Z` | `VALID` |
| `sess-d0428ec1` | `Software Engineer` | `DONE` | `d0428ec` | `2026-10-25T20:29:20Z` | `VALID` |
| `sess-cb8a691b` | `TDD Craftsman` | `RED` | `cb8a691` | `2026-11-01T13:43:34Z` | `VALID` |
| `sess-5b500d3d` | `TDD Craftsman` | `GREEN` | `5b500d3` | `2026-11-01T13:48:21Z` | `VALID` |
| `sess-29ad1a97` | `TDD Craftsman` | `REFACTOR` | `29ad1a9` | `2026-11-01T13:49:03Z` | `VALID` |
| `sess-4791d8dd` | `Software Engineer` | `DONE` | `4791d8d` | `2026-11-01T13:51:30Z` | `VALID` |
| `sess-13cff77e` | `TDD Craftsman` | `RED` | `13cff77` | `2026-11-01T21:26:55Z` | `VALID` |
| `sess-c88c37bf` | `TDD Craftsman` | `GREEN` | `c88c37b` | `2026-11-01T21:28:26Z` | `VALID` |
| `sess-e016e08d` | `TDD Craftsman` | `REFACTOR` | `e016e08` | `2026-11-01T21:30:52Z` | `VALID` |
| `sess-6830743a` | `TDD Craftsman` | `RED` | `6830743` | `2026-11-01T21:46:42Z` | `VALID` |
| `sess-f297ad6e` | `TDD Craftsman` | `GREEN` | `f297ad6` | `2026-11-01T21:51:23Z` | `VALID` |
| `sess-8417e79d` | `System Architect` | `SPEC` | `8417e79` | `2026-11-02T05:59:35Z` | `VALID` |
| `sess-058933a5` | `TDD Craftsman` | `RED` | `058933a` | `2026-11-02T06:02:57Z` | `VALID` |
| `sess-23386a2d` | `TDD Craftsman` | `GREEN` | `23386a2` | `2026-11-02T06:04:32Z` | `VALID` |
| `sess-e3fb756b` | `TDD Craftsman` | `REFACTOR` | `e3fb756` | `2026-11-02T06:05:37Z` | `VALID` |
| `sess-9630313f` | `Software Engineer` | `DONE` | `9630313` | `2026-11-02T06:16:21Z` | `VALID` |
| `sess-3c8efbb5` | `System Architect` | `SPEC` | `3c8efbb` | `2026-11-07T16:30:08Z` | `VALID` |
| `sess-c1b1ac18` | `TDD Craftsman` | `RED` | `c1b1ac1` | `2026-11-07T16:57:31Z` | `VALID` |
| `sess-e89a848e` | `TDD Craftsman` | `GREEN` | `e89a848` | `2026-11-07T17:10:39Z` | `VALID` |
| `sess-a99fddcc` | `TDD Craftsman` | `REFACTOR` | `a99fddc` | `2026-11-07T17:12:33Z` | `VALID` |
| `sess-04f946ba` | `Software Engineer` | `DONE` | `04f946b` | `2026-11-07T17:14:37Z` | `VALID` |
| `sess-9864551f` | `TDD Craftsman` | `RED` | `9864551` | `2026-11-07T20:27:43Z` | `VALID` |
| `sess-6392a887` | `TDD Craftsman` | `GREEN` | `6392a88` | `2026-11-07T20:30:02Z` | `VALID` |
| `sess-914e0f56` | `TDD Craftsman` | `REFACTOR` | `914e0f5` | `2026-11-07T20:30:43Z` | `VALID` |
| `sess-4e1140b4` | `Software Engineer` | `DONE` | `4e1140b` | `2026-11-07T20:34:42Z` | `VALID` |
| `sess-42338d07` | `TDD Craftsman` | `RED` | `42338d0` | `2026-11-07T21:07:20Z` | `VALID` |
| `sess-299280fb` | `TDD Craftsman` | `GREEN` | `299280f` | `2026-11-07T21:08:36Z` | `VALID` |
| `sess-14cf3261` | `TDD Craftsman` | `REFACTOR` | `14cf326` | `2026-11-07T21:08:46Z` | `VALID` |
| `sess-a9ab39e8` | `Software Engineer` | `DONE` | `a9ab39e` | `2026-11-07T21:10:12Z` | `VALID` |
| `sess-9c6a3dd8` | `TDD Craftsman` | `RED` | `9c6a3dd` | `2026-11-07T21:23:30Z` | `VALID` |
| `sess-369239dc` | `TDD Craftsman` | `GREEN` | `369239d` | `2026-11-07T21:24:12Z` | `VALID` |
| `sess-b85f7aa0` | `TDD Craftsman` | `REFACTOR` | `b85f7aa` | `2026-11-07T21:24:17Z` | `VALID` |
| `sess-3e6c2d6c` | `Software Engineer` | `DONE` | `3e6c2d6` | `2026-11-07T21:25:40Z` | `VALID` |
| `sess-09d4cde5` | `System Architect` | `SPEC` | `09d4cde` | `2026-11-08T10:35:58Z` | `VALID` |
| `sess-83796de5` | `TDD Craftsman` | `RED` | `83796de` | `2026-11-08T10:38:43Z` | `VALID` |
| `sess-ee49883c` | `TDD Craftsman` | `GREEN` | `ee49883` | `2026-11-08T10:45:18Z` | `VALID` |
| `sess-b98d084b` | `TDD Craftsman` | `REFACTOR` | `b98d084` | `2026-11-08T10:46:28Z` | `VALID` |
| `sess-4a6f2417` | `Software Engineer` | `DONE` | `4a6f241` | `2026-11-08T10:54:42Z` | `VALID` |
| `sess-c66898cd` | `System Architect` | `SPEC` | `c66898c` | `2026-11-08T10:58:26Z` | `VALID` |
| `sess-2fbdfbf9` | `TDD Craftsman` | `RED` | `2fbdfbf` | `2026-11-08T10:58:40Z` | `VALID` |
| `sess-f1cc49de` | `TDD Craftsman` | `GREEN` | `f1cc49d` | `2026-11-08T10:59:08Z` | `VALID` |
| `sess-7b9886b9` | `TDD Craftsman` | `REFACTOR` | `7b9886b` | `2026-11-08T10:59:15Z` | `VALID` |
| `sess-7f935cb6` | `Software Engineer` | `DONE` | `7f935cb` | `2026-11-08T11:00:39Z` | `VALID` |
| `sess-bfa7d2a0` | `System Architect` | `SPEC` | `bfa7d2a` | `2026-11-08T11:07:53Z` | `VALID` |
| `sess-9cb0fc68` | `TDD Craftsman` | `RED` | `9cb0fc6` | `2026-11-08T11:08:23Z` | `VALID` |
| `sess-f91e9316` | `TDD Craftsman` | `GREEN` | `f91e931` | `2026-11-08T11:10:00Z` | `VALID` |
| `sess-ece68b5c` | `TDD Craftsman` | `REFACTOR` | `ece68b5` | `2026-11-08T11:10:17Z` | `VALID` |
| `sess-779fc9b4` | `Software Engineer` | `DONE` | `779fc9b` | `2026-11-08T11:12:05Z` | `VALID` |
| `sess-a3b59688` | `System Architect` | `SPEC` | `a3b5968` | `2026-11-08T11:22:04Z` | `VALID` |
| `sess-f5977393` | `TDD Craftsman` | `RED` | `f597739` | `2026-11-08T11:22:33Z` | `VALID` |
| `sess-39e55163` | `TDD Craftsman` | `GREEN` | `39e5516` | `2026-11-08T11:25:12Z` | `VALID` |
| `sess-5788ad0d` | `TDD Craftsman` | `REFACTOR` | `5788ad0` | `2026-11-08T11:25:24Z` | `VALID` |
| `sess-98f9a33f` | `Software Engineer` | `DONE` | `98f9a33` | `2026-11-08T11:29:39Z` | `VALID` |
| `sess-dc6456b7` | `System Architect` | `SPEC` | `dc6456b` | `2026-11-08T11:45:16Z` | `VALID` |
| `sess-805948b8` | `TDD Craftsman` | `RED` | `805948b` | `2026-11-08T11:45:47Z` | `VALID` |
| `sess-4f3d14c5` | `TDD Craftsman` | `GREEN` | `4f3d14c` | `2026-11-08T11:49:09Z` | `VALID` |
| `sess-8a7c7d00` | `TDD Craftsman` | `REFACTOR` | `8a7c7d0` | `2026-11-08T11:49:24Z` | `VALID` |
| `sess-75662c01` | `Software Engineer` | `DONE` | `75662c0` | `2026-11-08T11:51:59Z` | `VALID` |
| `sess-73ccd57c` | `System Architect` | `SPEC` | `73ccd57` | `2026-11-08T11:55:44Z` | `VALID` |
| `sess-b4e22626` | `TDD Craftsman` | `RED` | `b4e2262` | `2026-11-08T11:56:25Z` | `VALID` |
| `sess-1045ab7b` | `TDD Craftsman` | `GREEN` | `1045ab7` | `2026-11-08T11:57:32Z` | `VALID` |
| `sess-7a9bd92a` | `TDD Craftsman` | `REFACTOR` | `7a9bd92` | `2026-11-08T11:57:46Z` | `VALID` |
| `sess-6132745a` | `Software Engineer` | `DONE` | `6132745` | `2026-11-08T11:58:44Z` | `VALID` |
| `sess-ac4b1ae4` | `System Architect` | `SPEC` | `ac4b1ae` | `2026-11-08T12:00:08Z` | `VALID` |
| `sess-f06083ce` | `TDD Craftsman` | `RED` | `f06083c` | `2026-11-08T12:00:26Z` | `VALID` |
| `sess-09e1fde6` | `TDD Craftsman` | `GREEN` | `09e1fde` | `2026-11-08T12:01:28Z` | `VALID` |
| `sess-170f6508` | `TDD Craftsman` | `REFACTOR` | `170f650` | `2026-11-08T12:01:52Z` | `VALID` |
| `sess-42745d1f` | `Software Engineer` | `DONE` | `42745d1` | `2026-11-08T12:02:51Z` | `VALID` |
| `sess-5efbd3f5` | `System Architect` | `SPEC` | `5efbd3f` | `2026-11-08T13:55:22Z` | `VALID` |
| `sess-8abd7c6f` | `TDD Craftsman` | `RED` | `8abd7c6` | `2026-11-08T13:57:50Z` | `VALID` |
| `sess-7eb2d1cc` | `TDD Craftsman` | `GREEN` | `7eb2d1c` | `2026-11-08T14:09:17Z` | `VALID` |
| `sess-0cb16c60` | `Software Engineer` | `DONE` | `0cb16c6` | `2026-11-08T14:11:10Z` | `VALID` |
| `sess-b0b6b0b8` | `System Architect` | `SPEC` | `b0b6b0b` | `2026-11-08T14:18:25Z` | `VALID` |
| `sess-80829e9c` | `TDD Craftsman` | `RED` | `80829e9` | `2026-11-08T14:19:13Z` | `VALID` |
| `sess-328c0dfa` | `TDD Craftsman` | `GREEN` | `328c0df` | `2026-11-08T14:24:34Z` | `VALID` |
| `sess-d608942a` | `Software Engineer` | `DONE` | `d608942` | `2026-11-08T14:38:28Z` | `VALID` |
| `sess-ee1eb4d4` | `System Architect` | `SPEC` | `ee1eb4d` | `2026-11-08T18:50:05Z` | `VALID` |
| `sess-2de532b7` | `TDD Craftsman` | `RED` | `2de532b` | `2026-11-08T18:51:11Z` | `VALID` |
| `sess-756d2e9a` | `TDD Craftsman` | `GREEN` | `756d2e9` | `2026-11-08T18:57:19Z` | `VALID` |
| `sess-124272b5` | `TDD Craftsman` | `REFACTOR` | `124272b` | `2026-11-08T19:03:19Z` | `VALID` |
| `sess-75363747` | `Software Engineer` | `DONE` | `7536374` | `2026-11-08T19:03:52Z` | `VALID` |
| `sess-59701644` | `Software Engineer` | `DONE` | `5970164` | `2026-11-08T20:01:05Z` | `VALID` |
| `sess-a1b0ce20` | `System Architect` | `SPEC` | `a1b0ce2` | `2026-11-08T20:01:30Z` | `VALID` |
| `sess-b6e47a63` | `TDD Craftsman` | `RED` | `b6e47a6` | `2026-11-08T20:02:03Z` | `VALID` |
| `sess-dcf2a26a` | `TDD Craftsman` | `GREEN` | `dcf2a26` | `2026-11-08T20:03:23Z` | `VALID` |
| `sess-ac6470c3` | `TDD Craftsman` | `REFACTOR` | `ac6470c` | `2026-11-08T20:03:28Z` | `VALID` |
| `sess-1da042d4` | `Software Engineer` | `DONE` | `1da042d` | `2026-11-08T20:08:20Z` | `VALID` |
| `sess-9d4cae5f` | `System Architect` | `SPEC` | `9d4cae5` | `2026-11-08T20:24:47Z` | `VALID` |
| `sess-6c96f37f` | `TDD Craftsman` | `RED` | `6c96f37` | `2026-11-08T20:25:15Z` | `VALID` |
| `sess-1c88beb6` | `TDD Craftsman` | `GREEN` | `1c88beb` | `2026-11-08T20:26:01Z` | `VALID` |
| `sess-f12003e4` | `TDD Craftsman` | `REFACTOR` | `f12003e` | `2026-11-08T20:26:06Z` | `VALID` |
| `sess-94a410e0` | `Software Engineer` | `DONE` | `94a410e` | `2026-11-08T20:30:04Z` | `VALID` |
| `sess-104d6609` | `System Architect` | `SPEC` | `104d660` | `2026-11-08T20:39:54Z` | `VALID` |
| `sess-01629edd` | `TDD Craftsman` | `RED` | `01629ed` | `2026-11-08T20:40:17Z` | `VALID` |
| `sess-5190370e` | `Software Engineer` | `DONE` | `5190370` | `2026-11-08T20:57:01Z` | `VALID` |
| `sess-c76308a5` | `System Architect` | `SPEC` | `c76308a` | `2026-11-08T21:11:02Z` | `VALID` |
| `sess-4d30c2e7` | `TDD Craftsman` | `RED` | `4d30c2e` | `2026-11-08T21:15:10Z` | `VALID` |

---

### 🧪 Test Provenance & Born-Green Verification

| Test Identifier | Status | Baseline Failure Digest | Verified At |
|---|---|---|---|
| `test_ast_scans_struct_fields_enum_variants_and_relationships` | `RedCheckpointVerified` | `9b8cab0c4e31c6b7...` | `2026-11-08T22:15:10+01:00` |
| `test_ast_scans_doc_commented_struct_fields_enum_variants_and_relationships` | `RedCheckpointVerified` | `9b8cab0c4e31c6b7...` | `2026-11-08T22:15:10+01:00` |
| `test_conceptual_mermaid_filters_errors_and_options_while_detailed_keeps_them` | `RedCheckpointVerified` | `9b8cab0c4e31c6b7...` | `2026-11-08T22:15:10+01:00` |
| `test_ast_scans_trait_implementation_realization_and_strong_enums` | `RedCheckpointVerified` | `9b8cab0c4e31c6b7...` | `2026-11-08T22:15:10+01:00` |
| `test_ast_shadowing_immunity_prevents_test_mocks_from_overwriting_domain_models` | `RedCheckpointVerified` | `9b8cab0c4e31c6b7...` | `2026-11-08T22:15:10+01:00` |
| `test_xgauntlet_repo_conceptual_domain_model_relationships` | `RedCheckpointVerified` | `9b8cab0c4e31c6b7...` | `2026-11-08T22:15:10+01:00` |
| `test_extract_declared_versions_includes_plugin_and_policy` | `RedCheckpointVerified` | `24d16e87708d41a2...` | `2026-11-08T15:19:13+01:00` |
| `test_update_manifest_versions_bumps_plugin_and_policy` | `RedCheckpointVerified` | `24d16e87708d41a2...` | `2026-11-08T15:19:13+01:00` |
| `test_workspace_manifests_synchronized_to_version_0_13_2` | `RedCheckpointVerified` | `24d16e87708d41a2...` | `2026-11-08T15:19:13+01:00` |
| `test_npm_cli_readme_identical_to_root_readme` | `RedCheckpointVerified` | `24d16e87708d41a2...` | `2026-11-08T15:19:13+01:00` |
| `test_architecture_config_parsing_from_toml` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_passes_on_compliant_workspace` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_fails_on_monolithic_production_file` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_fails_on_monolithic_test_file` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_honors_explicit_exemption` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_fails_if_exemption_ceiling_exceeded` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_catches_forbidden_tokens` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_catches_forbidden_imports` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_fails_when_exemption_lacks_adr` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_fails_when_exemption_adr_not_found` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_check_fails_when_exemption_adr_is_not_accepted` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_architecture_config_parsing_from_toml_with_adr` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_gate_b_1_level_slice_sibling_and_glob_exemption` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_mermaid_class_diagram_generation_conceptual` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_mermaid_class_diagram_detailed_includes_technical_structs` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_glossary_drift_detects_orphan_models_missing_in_context` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_glossary_drift_passes_when_all_models_defined_in_context` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_glossary_drift_detects_prohibited_avoid_synonyms_in_fields` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_check_spec_supports_1_level_slice_without_subdomain` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_glossary_drift_supports_parenthetical_aliases_and_dogfoods_workspace` | `RedCheckpointVerified` | `6dfbcc2100981406...` | `2026-11-08T14:57:50+01:00` |
| `test_workspace_manifests_synchronized_to_version_0_13_1` | `RedCheckpointVerified` | `df6785ad3278087b...` | `2026-11-08T13:00:26+01:00` |
| `test_workspace_manifests_synchronized_to_version_0_13_0` | `RedCheckpointVerified` | `13b3f4cc8f63845d...` | `2026-11-08T12:56:25+01:00` |
| `test_paths_config_default_diagram_file` | `RedCheckpointVerified` | `a00ae1cd4b5d35aa...` | `2026-11-08T12:45:47+01:00` |
| `test_verify_save_generates_architecture_mermaid` | `RedCheckpointVerified` | `a00ae1cd4b5d35aa...` | `2026-11-08T12:45:47+01:00` |
| `test_verify_save_skips_when_diagram_file_empty` | `RedCheckpointVerified` | `a00ae1cd4b5d35aa...` | `2026-11-08T12:45:47+01:00` |
| `test_safe_non_destructive_guarantee_preserves_existing_files` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_force_flag_overwrites_existing_files` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_dry_run_does_not_modify_disk` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_stack_detection_and_custom_stacks` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_invalid_stack_rejected` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_generated_templates_pass_check_spec_and_config_validation` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_all_supported_stacks_generate_valid_configs` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_cli_init_integration` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_scaffold_agents_md_contains_dynamic_cockpit_hud_and_checkpoint_protocol` | `RedCheckpointVerified` | `4e4417bb01a5dce5...` | `2026-11-08T12:08:23+01:00` |
| `test_scaffold_gauntlet_toml_contains_architecture_guidance` | `RedCheckpointVerified` | `6c0e45497d42950a...` | `2026-11-08T11:58:40+01:00` |
| `test_readme_documents_architecture_gates_and_safe_mode` | `RedCheckpointVerified` | `6c0e45497d42950a...` | `2026-11-08T11:58:40+01:00` |
| `test_readme_documents_brownfield_migration_blueprint` | `RedCheckpointVerified` | `6c0e45497d42950a...` | `2026-11-08T11:58:40+01:00` |
| `test_ac1_architecture_config_toml_roundtrip_for_layers_slices_and_exemptions` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_ac2_gate_a_layer_direction_violations_across_rust_ts_and_java` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_ac2_gate_b_cross_slice_isolation_and_internal_slice_freedom` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_ac2_gate_c_clean_core_forbids_ui_framework_imports` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_ac3_domain_slice_task_spec_ref_validation_and_done_exemption` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_ac4_adr_lifecycle_statuses_and_release_coverage_filtering` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_ac5_sovereign_skills_and_coding_standards_document_slice_taxonomy_and_polyglot_examples` | `RedCheckpointVerified` | `046a1345f774cbae...` | `2026-11-07T17:57:31+01:00` |
| `test_concurrent_worktree_inverse_preflight_and_nonce_uniqueness` | `RedCheckpointVerified` | `1d6dcd40465bcc03...` | `2026-11-02T07:02:57+01:00` |
| `test_macos_style_symlinked_temp_and_workspace_parity` | `RedCheckpointVerified` | `1d6dcd40465bcc03...` | `2026-11-02T07:02:57+01:00` |
| `test_codebase_free_of_collision_prone_temp_paths_and_unix_only_env` | `RedCheckpointVerified` | `1d6dcd40465bcc03...` | `2026-11-02T07:02:57+01:00` |
| `test_no_unguarded_top_level_imports_in_platform_gated_tests` | `RedCheckpointVerified` | `1d6dcd40465bcc03...` | `2026-11-02T07:02:57+01:00` |
| `test_harness_whitelist_paths` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_session_brain_artifact_paths` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_windows_userprofile_and_cross_platform_path_normalization` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_backslash_traversal_rejected_across_platforms` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_hook_run_command_remains_strictly_gatekept` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_hook_parameters_outside_workspace_denied_with_path` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_structured_stderr_on_claude_code_and_malformed_stdin` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_hook_read_only_tools_allowed_in_spec_phase` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_rust_stack_profile_deduplication` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_hook_blocks_command_injection_and_unscoped_git_commit_without_task` | `RedCheckpointVerified` | `5971e55478f0a410...` | `2026-11-01T22:26:55+01:00` |
| `test_rust_target_resolver_standalone_integration_test` | `RedCheckpointVerified` | `03220af534fef745...` | `2026-11-01T18:40:44+01:00` |
| `test_rust_target_resolver_harness_submodule` | `RedCheckpointVerified` | `03220af534fef745...` | `2026-11-01T18:40:44+01:00` |
| `test_rust_target_resolver_lib_unit_test` | `RedCheckpointVerified` | `03220af534fef745...` | `2026-11-01T18:40:44+01:00` |
| `test_fast_track_eligibility_criteria` | `RedCheckpointVerified` | `03220af534fef745...` | `2026-11-01T18:40:44+01:00` |
| `test_fast_track_worktree_inverse_proof_born_green_rejection` | `RedCheckpointVerified` | `03220af534fef745...` | `2026-11-01T18:40:44+01:00` |
| `test_always_passes` | `RedCheckpointVerified` | `03220af534fef745...` | `2026-11-01T18:40:44+01:00` |
| `test_antigravity_2_6_tools_normalization` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_antigravity_2_6_hook_handler_protocol` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_antigravity_resolve_effective_workspace_and_containment` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_antigravity_workspace_containment_denies_outside_repo` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_manifest_version_extraction_and_harmony` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_manifest_version_mismatch_detection` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_missing_changelog_detection` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_changelog_version_matching_and_allow_unreleased` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_adr_coverage_detection` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_changelog_parser_helper` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_real_workspace_release_readiness` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_cli_check_release_integration` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_unformatted_code_detection_in_release_gatekeeper` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_wasm_policy_version_mismatch_detection_in_release_gatekeeper` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_format_gate_fails_closed_when_formatter_missing` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_wasm_policy_fails_closed_on_corrupt_bytecode` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_strict_release_gate_fails_closed_when_evidence_report_missing` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_package_json_key_order_preservation_on_update` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_in_flight_staging_skips_dirty_worktree_but_enforces_strict_verification` | `RedCheckpointVerified` | `fc39acdeb3956d58...` | `2026-11-01T14:43:34+01:00` |
| `test_ratchet_enforcement_rejects_unauthorized_ceiling_inflation` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_ratchet_enforcement_accepts_authorized_ceiling_inflation` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_ratchet_enforcement_rejects_inflation_exceeding_authorized_limit` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_ratchet_enforcement_allows_lowered_or_unchanged_ceilings` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_ratchet_enforcement_allows_new_exemption_with_valid_adr` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_ratchet_enforcement_skips_in_non_git_workspace_with_notice` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_ratchet_enforcement_fails_closed_when_git_baseline_unreadable` | `RedCheckpointVerified` | `b62800e1576998b6...` | `2026-10-25T21:21:37+01:00` |
| `test_migration_injects_canonical_telemetry_format` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_migration_normalizes_defective_harness_hook_and_is_idempotent` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_migration_normalizes_legacy_harness_antigravity` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_migration_merges_simultaneous_legacy_and_modern_groups_without_data_loss` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_doctor_fix_reports_migration_errors_truthfully` | `RedCheckpointVerified` | `845eb4d02ba21f7d...` | `2026-10-25T20:34:52+01:00` |
| `test_preflight_leaves_working_tree_untouched_during_test_run` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_runs_in_isolated_worktree_and_cleans_up` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_does_not_resurrect_deleted_production_file` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_baseline_excludes_untracked_production_files` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_preflight_fails_closed_without_head` | `RedCheckpointVerified` | `f0c1287911ae91dd...` | `2026-10-25T20:19:28+01:00` |
| `test_doctor_warns_on_legacy_hooks_namespace_and_missing_preinvocation` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_doctor_warns_on_legacy_evidence_file_in_gauntlet_toml` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_doctor_passes_when_hooks_and_evidence_are_modern` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_doctor_fix_flag_auto_remediates_governance_issues` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_dry_run_does_not_modify_disk` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_modernizes_hooks_preserving_custom_matchers` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_modernizes_gauntlet_toml_preserving_custom_layers` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_migration_engine_scaffolds_missing_adr_template` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_runtime_backward_compatibility_with_legacy_hooks_namespace` | `RedCheckpointVerified` | `f18b5278ca4d2b25...` | `2026-10-25T17:47:22+01:00` |
| `test_session_lineage_trail_genesis_and_chaining` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_session_lineage_tamper_detection` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_verification_report_backward_compatibility` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_evidence_markdown_renders_lineage_table` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_checkpoint_engine_appends_lineage` | `RedCheckpointVerified` | `9e57b6c459f4004b...` | `2026-10-25T15:00:27+01:00` |
| `test_adr_with_complete_okf_header_validates_ok` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_without_generated_gives_generated_missing_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_with_invalid_actor_gives_invalid_actor_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_with_invalid_timestamp_gives_invalid_timestamp_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_adr_with_missing_verified_gives_warning_not_error` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_check_all_adrs_skips_template_and_readme` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_scaffold_template_includes_generated_and_verified` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_xgauntlet_own_adrs_validate_cleanly` | `RedCheckpointVerified` | `808853cea2e5c9a3...` | `2026-10-25T17:01:02+01:00` |
| `test_models_and_conventions` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_compose_commit_message_variations` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_checkpoint_fails_without_active_task` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_spec_phase_checkpoint` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_red_phase_fails_when_test_passes` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_red_phase_succeeds_when_test_fails` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_autonomous_red_to_green_transition_checkpoint` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_green_phase_fails_when_test_fails` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_green_phase_succeeds_when_test_passes` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_refactor_phase_succeeds_when_green` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_skip_verify_override_bypasses_preflight` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_done_phase_succeeds_and_creates_chore_commit` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_cli_checkpoint_red_and_green_invariants` | `RedCheckpointVerified` | `a8f350e1e23e3240...` | `2026-10-25T16:22:47+01:00` |
| `test_extract_test_identifiers_polyglot` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_synchronous_case` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_asynchronous_worker` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_inline_definition` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_detect_born_green_tests_in_git_diff` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_initial_requirement` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_sneaky_born_green_requirement` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_inverse_preflight_detects_tautological_test` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_inverse_preflight_verifies_legitimate_test` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_checkpoint_green_fails_closed_on_born_green_with_strict_provenance` | `RedCheckpointVerified` | `b02ab9f2e3aea63d...` | `2026-10-25T15:50:03+01:00` |
| `test_collect_task_test_provenance_and_verification_report` | `InversePreflightVerified` | `c46f890b2d9b738e...` | `2026-10-25T15:07:56Z` |
| `test_doctor_warns_on_tsc_no_emit_in_composite_typescript_repo` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_doctor_passes_when_composite_typescript_uses_tsc_b` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_detect_composite_typescript_tsconfig_content` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_detect_composite_typescript_in_workspace` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_scaffold_init_composite_typescript_generates_tsc_b` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_scaffold_init_standard_typescript_generates_tsc_no_emit` | `RedCheckpointVerified` | `ccdb618c81ed206d...` | `2026-10-25T17:21:56+01:00` |
| `test_canonical_pre_invocation_cmd_single_source_of_truth` | `InversePreflightVerified` | `e63e0e0ec71fb4c6...` | `2026-10-25T19:39:49Z` |

---
