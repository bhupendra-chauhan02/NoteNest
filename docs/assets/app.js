(function () {
  const modal = document.querySelector('[data-modal]');
  if (modal) {
    const closeBtn = modal.querySelector('[data-modal-close]');
    const openButtons = document.querySelectorAll('[data-modal-open]');
    const closeModal = () => modal.classList.remove('is-open');

    openButtons.forEach((btn) => {
      btn.addEventListener('click', () => modal.classList.add('is-open'));
    });

    if (closeBtn) {
      closeBtn.addEventListener('click', closeModal);
    }

    modal.addEventListener('click', (event) => {
      if (event.target === modal) {
        closeModal();
      }
    });
  }

  const demoRoot = document.querySelector('[data-demo-root]');
  if (!demoRoot) {
    return;
  }

  const fileInput = demoRoot.querySelector('#note-file');
  const textInput = demoRoot.querySelector('#note-text');
  const sampleSelect = demoRoot.querySelector('#sample-note');
  const placeholderSelect = demoRoot.querySelector('#placeholder-style');
  const placeholderLegend = demoRoot.querySelector('#placeholder-legend');
  const statusOutput = demoRoot.querySelector('#status-output');
  const protectedOutput = demoRoot.querySelector('#protected-output');
  const patientOutput = demoRoot.querySelector('#patient-output');
  const clinicianOutput = demoRoot.querySelector('#clinician-output');
  const coverageOutput = demoRoot.querySelector('#coverage-output');
  const phiSummary = demoRoot.querySelector('#phi-summary');
  const phiFlags = demoRoot.querySelector('#phi-flags');
  const clearBtn = demoRoot.querySelector('#clear-btn');
  const toast = document.querySelector('#toast');

  const convertButtons = [
    demoRoot.querySelector('#convert-btn'),
    demoRoot.querySelector('#convert-btn-secondary'),
  ].filter(Boolean);

  const tabButtons = demoRoot.querySelectorAll('[data-tab-target]');
  const tabPanels = demoRoot.querySelectorAll('[data-tab-panel]');

  const modeButtons = demoRoot.querySelectorAll('[data-mode]');
  const modePanels = demoRoot.querySelectorAll('[data-mode-panel]');
  let activeMode = 'summarize';

  const clinicianButtons = demoRoot.querySelectorAll('[data-clinician]');
  let activeClinician = 'soap';
  let lastClinicianViews = null;

  const setActiveTab = (target) => {
    tabButtons.forEach((btn) => {
      const isActive = btn.dataset.tabTarget === target;
      btn.setAttribute('aria-selected', String(isActive));
    });

    tabPanels.forEach((panel) => {
      panel.hidden = panel.dataset.tabPanel !== target;
    });
  };

  const setMode = (mode) => {
    activeMode = mode;
    modeButtons.forEach((btn) => {
      const isActive = btn.dataset.mode === mode;
      btn.setAttribute('aria-selected', String(isActive));
    });

    modePanels.forEach((panel) => {
      panel.hidden = panel.dataset.modePanel !== mode;
    });
  };

  const setClinician = (mode) => {
    activeClinician = mode;
    clinicianButtons.forEach((btn) => {
      const isActive = btn.dataset.clinician === mode;
      btn.setAttribute('aria-selected', String(isActive));
    });
    if (lastClinicianViews && clinicianOutput) {
      clinicianOutput.textContent = renderClinicianView(lastClinicianViews, activeClinician);
    }
  };

  tabButtons.forEach((btn) => {
    btn.addEventListener('click', () => setActiveTab(btn.dataset.tabTarget));
  });

  modeButtons.forEach((btn) => {
    btn.addEventListener('click', () => setMode(btn.dataset.mode));
  });

  clinicianButtons.forEach((btn) => {
    btn.addEventListener('click', () => setClinician(btn.dataset.clinician));
  });

  setActiveTab('patient');
  setMode('summarize');
  setClinician('soap');

  const showToast = (message) => {
    if (!toast) {
      return;
    }
    toast.textContent = message;
    toast.classList.add('is-visible');
    setTimeout(() => toast.classList.remove('is-visible'), 3000);
  };

  const setLoading = (isLoading) => {
    convertButtons.forEach((btn) => {
      btn.disabled = isLoading;
      btn.dataset.loading = isLoading ? 'true' : 'false';
      btn.textContent = isLoading ? 'Processing...' : 'Convert';
    });
  };

  const copyButtons = demoRoot.querySelectorAll('[data-copy-target]');
  copyButtons.forEach((btn) => {
    btn.dataset.copyLabel = btn.textContent;
    btn.addEventListener('click', async () => {
      const targetId = btn.dataset.copyTarget;
      const targetEl = demoRoot.querySelector(`#${targetId}`);
      if (!targetEl) {
        return;
      }

      const text = targetEl.textContent || '';
      try {
        await navigator.clipboard.writeText(text);
        btn.textContent = 'Copied';
        setTimeout(() => {
          btn.textContent = btn.dataset.copyLabel || 'Copy';
        }, 1200);
      } catch (error) {
        showToast('Copy failed.');
      }
    });
  });

  const downloadButtons = demoRoot.querySelectorAll('[data-download-target]');
  downloadButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
      const targetId = btn.dataset.downloadTarget;
      const targetEl = demoRoot.querySelector(`#${targetId}`);
      if (!targetEl) {
        return;
      }
      const blob = new Blob([targetEl.textContent || ''], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = btn.dataset.downloadName || 'output.txt';
      document.body.appendChild(link);
      link.click();
      link.remove();
      URL.revokeObjectURL(url);
    });
  });

  const placeholderToken = (kind, style) => {
    if (style === 'angle') {
      return `<${kind}>`;
    }
    return `[${kind}_${style.toUpperCase()}]`;
  };

  const updateLegend = (style) => {
    if (!placeholderLegend) {
      return;
    }
    const example = placeholderToken('EMAIL', style);
    placeholderLegend.textContent = `Sensitive fields are replaced with placeholders (e.g., ${example}). You can switch styles (Protected / Masked / Hidden / Removed / <TAG>).`;
  };

  const sampleNotes = {
    note1: `ER note (messy): JOHN O?? 43M chest tightness x2d, worse stairs + SOB. wife Mary 0176-12345678 called.\nemail john.osmith@gmail.com MRN 883920 DOB 12/03/1982 addr 12 Hauptstrasse 80331 Muenchen.\npmh HTN/DM2; meds metformin 500 bid + ramipril 5mg od; nkda.\nvitals BP168/96 HR108 T37.2. ecg ?st-depr. trop 0.08 ng/mL.\nplan: send ED; repeat trop 3h; ASA; consider heparin; f/u cardio.`,
    note2: `walk-in messy note: \"abdo pain??\" started last monday; worse after meals.\nstress @ work; sleeps 3-4h. denies vomiting; some diarrhea.\ncontact sara.khan@web.de +49 152 98765432 ID# AOK-1199-22.\nmeds unsure \"ibu sometimes\". allergy: penicillin rash.`,
    note3: `ED triage: 27 F dizzy + nausea since Tuesday. BP90/60 HR122 T38.1 SpO2 94.\nLabs: CRP 20 mg/L, WBC 14.2. CT pending.\nPlan: IV fluids, repeat labs in 4h, follow-up clinic.\nAddress: 55 Market Street, Springfield.`
  };

  const protectionRules = [
    {
      key: 'name',
      kind: 'NAME',
      regex: /\b(Name|Patient Name|Patient|Pt)\s*:\s*[A-Z][a-z]+(?:\s+[A-Z][a-z]+){1,2}/g,
      labeled: true,
    },
    {
      key: 'email',
      kind: 'EMAIL',
      regex: /[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi,
    },
    {
      key: 'phone',
      kind: 'PHONE',
      regex: /(?:\+?\d{1,2}\s*)?(?:\(?\d{3}\)?[\s.-]?)\d{3}[\s.-]?\d{4}/g,
    },
    {
      key: 'dob',
      kind: 'DOB',
      regex: /\b(DOB|Date of Birth)\s*[:\-]?\s*(\d{1,2}[\/\-]\d{1,2}[\/\-]\d{2,4}|\d{4}[\/\-]\d{1,2}[\/\-]\d{1,2})/gi,
      labeled: true,
    },
    {
      key: 'address_label',
      kind: 'ADDRESS',
      regex: /\bAddress\s*[:\-].*/gi,
      labelText: 'Address',
    },
    {
      key: 'address',
      kind: 'ADDRESS',
      regex: /\b\d{1,5}\s+[A-Za-z0-9.'-]+(?:\s+[A-Za-z0-9.'-]+){0,4}\s+(?:Street|St|Avenue|Ave|Road|Rd|Boulevard|Blvd|Lane|Ln|Drive|Dr|Court|Ct|Way|Place|Pl)\b/gi,
    },
    {
      key: 'id',
      kind: 'ID',
      regex: /\b(ID|MRN|Record|Account)\s*[:#]?\s*\d{5,}\b/gi,
      labeled: true,
    },
    {
      key: 'id_generic',
      kind: 'ID',
      regex: /\b\d{6,}\b/g,
    },
  ];

  const protectText = (text, style) => {
    const counts = {
      name: 0,
      email: 0,
      phone: 0,
      dob: 0,
      id: 0,
      address: 0,
      other: 0,
    };

    let protectedText = text;
    protectionRules.forEach((rule) => {
      protectedText = protectedText.replace(rule.regex, (match, label) => {
        if (rule.key === 'address_label' || rule.key === 'address') {
          counts.address += 1;
        } else if (rule.key === 'id' || rule.key === 'id_generic') {
          counts.id += 1;
        } else if (counts[rule.key] !== undefined) {
          counts[rule.key] += 1;
        }

        const token = placeholderToken(rule.kind, style);
        if (rule.labelText) {
          return `${rule.labelText}: ${token}`;
        }
        if (rule.labeled && label) {
          return `${label}: ${token}`;
        }
        return token;
      });
    });

    const flags = [];
    if (/(\d{8,})/.test(text)) {
      flags.push('Long digit sequence');
    }
    const lineHits = text.split(/\r?\n/).filter((line) => {
      let hits = 0;
      protectionRules.forEach((rule) => {
        if (rule.regex.test(line)) {
          hits += 1;
        }
      });
      return hits >= 2;
    });
    if (lineHits.length) {
      flags.push('Multiple identifiers on a line');
    }

    return { protectedText, counts, flags };
  };

  const normalizeList = (items) => {
    const cleaned = items.map((item) => item.trim()).filter(Boolean);
    return cleaned.length ? cleaned : ['Not found'];
  };

  const normalizeString = (value) => (value && value.trim() ? value.trim() : 'Not found');

  const extractSummary = (text) => {
    const summary = {
      chiefConcern: [],
      duration: [],
      symptoms: [],
      negatives: [],
      meds: [],
      allergies: [],
      vitals: [],
      tests: [],
      plan: [],
      context: [],
      concerns: [],
      coping: [],
    };

    const lines = text
      .split(/\r?\n/)
      .map((line) => line.trim())
      .filter(Boolean);

    const durationRe = /\b(x\s?\d+\s?(d|day|days|w|wk|week|weeks|mo|month|months)|for\s+\d+\s?(d|day|days|w|wk|week|weeks|mo|month|months)|since\s+\w+|started\s+last\s+\w+)\b/i;
    const vitalRe = /\b(BP\s?\d{2,3}\/\d{2,3}|HR\s?\d{2,3}|RR\s?\d{2,3}|T\s?\d{2}\.?\d?|Temp\s?\d{2}\.?\d?|SpO2\s?\d{2,3})\b/i;
    const testRe = /\b(ecg|ekg|trop|troponin|crp|hba1c|labs?|ct|cxr|x-ray)\b/i;
    const medRe = /\b(meds?|taking|metformin|ramipril|lisinopril|amlodipine|ibu|ibuprofen|asa)\b/i;
    const allergyRe = /\b(nkda|allergy|allergies|penicillin)\b/i;
    const symptomRe = /\b(sob|shortness of breath|breathe|breathing|dyspnea|cp|chest pain|tightness|abdo pain|abdominal pain|fatigue|nausea|vomiting|diarrhea|cough|fever|dizzy|dizziness|headache)\b/gi;

    const extractSymptoms = (line) => {
      const hits = [];
      let match;
      while ((match = symptomRe.exec(line)) !== null) {
        const token = match[0].toLowerCase();
        const normalized = token === 'sob' ? 'shortness of breath'
          : token === 'cp' ? 'chest pain'
          : token === 'abdo pain' ? 'abdominal pain'
          : token;
        if (!hits.includes(normalized)) {
          hits.push(normalized);
        }
      }
      return hits;
    };

    let current = null;

    const setFromHeading = (key, line, headingRegex) => {
      const match = line.match(headingRegex);
      if (match) {
        const rest = line.replace(headingRegex, '').trim();
        if (rest) {
          summary[key].push(rest);
        }
        current = key;
        return true;
      }
      return false;
    };

    for (const line of lines) {
      if (setFromHeading('chiefConcern', line, /^(Chief Complaint|Chief Concern|CC|Reason for Visit)[:\-]/i)) {
        continue;
      }
      if (setFromHeading('plan', line, /^(Plan|Treatment|Recommendations|Follow[- ]?Up)[:\-]/i)) {
        continue;
      }

      if (current === 'plan') {
        const cleaned = line.replace(/^[-*\d.]+\s*/, '').trim();
        if (cleaned) {
          summary.plan.push(cleaned);
        }
        continue;
      }

      const durationMatch = line.match(durationRe);
      if (durationMatch) {
        summary.duration.push(durationMatch[0]);
      }

      const symptoms = extractSymptoms(line);
      if (symptoms.length) {
        summary.symptoms.push(...symptoms);
      }

      if (/denies/i.test(line)) {
        summary.negatives.push(line);
      }

      const vitals = line.match(vitalRe);
      if (vitals) {
        summary.vitals.push(vitals[0]);
      }

      if (testRe.test(line)) {
        summary.tests.push(line);
      }

      if (medRe.test(line)) {
        summary.meds.push(line);
      }

      if (allergyRe.test(line)) {
        if (/nkda/i.test(line) || /no known drug allergies/i.test(line)) {
          summary.allergies = ['NKDA'];
        } else {
          summary.allergies.push(line);
        }
      }

      if (/stress|work|context/i.test(line)) {
        summary.context.push(line);
      }
      if (/concern|worried|denies/i.test(line)) {
        summary.concerns.push(line);
      }
      if (/sleep|coping/i.test(line)) {
        summary.coping.push(line);
      }
    }

    summary.chiefConcern = normalizeList(summary.chiefConcern);
    summary.duration = normalizeList(summary.duration);
    summary.symptoms = normalizeList(summary.symptoms);
    summary.negatives = normalizeList(summary.negatives);
    summary.meds = normalizeList(summary.meds);
    summary.allergies = normalizeList(summary.allergies);
    summary.vitals = normalizeList(summary.vitals);
    summary.tests = normalizeList(summary.tests);
    summary.plan = normalizeList(summary.plan);
    summary.context = normalizeList(summary.context);
    summary.concerns = normalizeList(summary.concerns);
    summary.coping = normalizeList(summary.coping);

    if (summary.chiefConcern[0] === 'Not found' && summary.symptoms[0] !== 'Not found') {
      summary.chiefConcern = [summary.symptoms[0]];
    }

    return summary;
  };

  const buildPatientView = (summary) => {
    const mainConcern = summary.chiefConcern[0] !== 'Not found'
      ? summary.chiefConcern.join('; ')
      : summary.symptoms[0] !== 'Not found'
        ? summary.symptoms.join('; ')
        : 'Symptoms not clearly stated';

    const triggers = summary.symptoms.filter((item) => /worse|stairs|exertion/i.test(item));

    return {
      main_concern: normalizeString(mainConcern),
      onset_duration: normalizeString(summary.duration.join(', ')),
      triggers: normalizeList(triggers),
      what_it_could_mean:
        'These symptoms can have many causes. Some need urgent evaluation when breathing or chest symptoms are present.',
      what_we_found: {
        symptoms: normalizeList(summary.symptoms),
        negatives: normalizeList(summary.negatives),
        medications: normalizeList(summary.meds),
        allergies: normalizeList(summary.allergies),
        tests_results: normalizeList(summary.tests),
        vitals: normalizeList(summary.vitals),
      },
      next_steps: normalizeList(
        summary.plan[0] !== 'Not found'
          ? summary.plan
          : ['Follow the plan and confirm timing with your clinician.']
      ),
      questions_to_ask: [
        'What is the most likely cause of my symptoms?',
        'What warning signs should make me seek help immediately?',
        'What tests are still pending, and what do they mean?',
        'What is my follow-up plan and timeline?',
      ],
      urgent_red_flags: [
        'Worsening chest pain or pressure',
        'Severe difficulty breathing',
        'Fainting or confusion',
        'Blue lips/face, or new weakness on one side',
      ],
      disclaimer: 'This summary is for informational use and does not replace medical advice.',
    };
  };

  const buildClinicianViews = (summary) => {
    const soap = {
      S: normalizeList([
        summary.chiefConcern[0] !== 'Not found' ? `CC: ${summary.chiefConcern.join('; ')}` : 'CC: Not found',
        summary.duration[0] !== 'Not found' ? `Duration: ${summary.duration.join('; ')}` : 'Duration: Not found',
        summary.symptoms[0] !== 'Not found' ? `Symptoms: ${summary.symptoms.join('; ')}` : 'Symptoms: Not found',
      ]),
      O: normalizeList([
        summary.vitals[0] !== 'Not found' ? `Vitals: ${summary.vitals.join('; ')}` : 'Vitals: Not found',
        summary.tests[0] !== 'Not found' ? `Tests: ${summary.tests.join('; ')}` : 'Tests: Not found',
        summary.meds[0] !== 'Not found' ? `Meds: ${summary.meds.join('; ')}` : 'Meds: Not found',
        summary.allergies[0] !== 'Not found' ? `Allergies: ${summary.allergies.join('; ')}` : 'Allergies: Not found',
      ]),
      A: normalizeList(['Assessment not explicitly stated.']),
      P: normalizeList(summary.plan),
    };

    const fiveCs = {
      chief_complaint: normalizeString(summary.chiefConcern.join('; ')),
      course: normalizeList(summary.duration),
      context: normalizeList(summary.context),
      concerns: normalizeList(summary.concerns),
      coping: normalizeList(summary.coping),
    };

    return { soap, fiveCs };
  };

  const formatSection = (title, items) => {
    const list = items.map((item) => `- ${item}`).join('\n');
    return `${title}\n${list}`;
  };

  const renderPatientView = (patient) => {
    const cameIn = [
      `Main concern: ${patient.main_concern}`,
      patient.onset_duration !== 'Not found' ? `Duration: ${patient.onset_duration}` : null,
      patient.triggers[0] !== 'Not found' ? `Triggers: ${patient.triggers.join('; ')}` : null,
    ].filter(Boolean);

    const found = [
      `Symptoms: ${patient.what_we_found.symptoms.join('; ')}`,
      `Negatives: ${patient.what_we_found.negatives.join('; ')}`,
      `Medications: ${patient.what_we_found.medications.join('; ')}`,
      `Allergies: ${patient.what_we_found.allergies.join('; ')}`,
      `Tests/results: ${patient.what_we_found.tests_results.join('; ')}`,
      `Vitals: ${patient.what_we_found.vitals.join('; ')}`,
    ];

    return [
      formatSection('What you came in with', cameIn),
      formatSection('What it could mean', [patient.what_it_could_mean]),
      formatSection('What we found in your note', found),
      formatSection('What to do next (checklist)', patient.next_steps),
      formatSection('Questions to ask your clinician', patient.questions_to_ask),
      formatSection('When to seek urgent care', patient.urgent_red_flags),
      formatSection('Disclaimer', [patient.disclaimer]),
    ].join('\n\n');
  };

  const renderClinicianView = (views, mode) => {
    if (mode === '5cs') {
      return [
        formatSection("5C's - Chief complaint", [views.fiveCs.chief_complaint]),
        formatSection("5C's - Course", views.fiveCs.course),
        formatSection("5C's - Context", views.fiveCs.context),
        formatSection("5C's - Concerns", views.fiveCs.concerns),
        formatSection("5C's - Coping", views.fiveCs.coping),
      ].join('\n\n');
    }
    return [
      formatSection('SOAP - S', views.soap.S),
      formatSection('SOAP - O', views.soap.O),
      formatSection('SOAP - A', views.soap.A),
      formatSection('SOAP - P', views.soap.P),
    ].join('\n\n');
  };

  const buildCoverage = (summary, counts) => {
    const fields = [
      ['chief_concern', summary.chiefConcern],
      ['duration', summary.duration],
      ['symptoms', summary.symptoms],
      ['meds', summary.meds],
      ['allergies', summary.allergies],
      ['vitals', summary.vitals],
      ['tests', summary.tests],
      ['plan', summary.plan],
      ['context', summary.context],
      ['concerns', summary.concerns],
      ['coping', summary.coping],
    ];
    let found = 0;
    const missing = [];
    fields.forEach(([label, items]) => {
      if (items.length === 1 && items[0] === 'Not found') {
        missing.push(label);
      } else {
        found += 1;
      }
    });
    return {
      fields_found: found,
      fields_missing: missing,
      protected_counts: counts,
    };
  };

  const renderCoverage = (coverage) => {
    return [
      `fields_found: ${coverage.fields_found}`,
      `missing: ${coverage.fields_missing.length ? coverage.fields_missing.join(', ') : 'none'}`,
      `protected_counts: names ${coverage.protected_counts.name}, phones ${coverage.protected_counts.phone}, emails ${coverage.protected_counts.email}, dobs ${coverage.protected_counts.dob}, ids ${coverage.protected_counts.id}, addresses ${coverage.protected_counts.address}`,
    ].join('\n');
  };

  const parseInputText = async () => {
    const rawText = textInput.value.trim();
    if (rawText) {
      return rawText;
    }

    if (!fileInput.files.length) {
      throw new Error('Add a note by pasting text or selecting a file.');
    }

    const file = fileInput.files[0];
    const content = await file.text();
    if (file.name.toLowerCase().endsWith('.json')) {
      try {
        const parsed = JSON.parse(content);
        return parsed.note || parsed.text || parsed.content || content;
      } catch (error) {
        return content;
      }
    }

    return content;
  };

  const handleConvert = async () => {
    setLoading(true);
    if (statusOutput) {
      statusOutput.textContent = 'Processing';
    }

    try {
      const inputText = await parseInputText();
      if (!inputText.trim()) {
        throw new Error('Input is empty.');
      }

      const style = placeholderSelect ? placeholderSelect.value : 'protected';
      updateLegend(style);

      const { protectedText, counts, flags } = protectText(inputText, style);
      const summary = extractSummary(protectedText);
      const patientView = buildPatientView(summary);
      const clinicianViews = buildClinicianViews(summary);
      const coverage = buildCoverage(summary, counts);

      protectedOutput.textContent = protectedText;
      patientOutput.textContent = renderPatientView(patientView);
      lastClinicianViews = clinicianViews;
      clinicianOutput.textContent = renderClinicianView(clinicianViews, activeClinician);
      if (coverageOutput) {
        coverageOutput.textContent = renderCoverage(coverage);
      }

      if (phiSummary) {
        const lines = [
          `name: ${counts.name}`,
          `email: ${counts.email}`,
          `phone: ${counts.phone}`,
          `dob: ${counts.dob}`,
          `id: ${counts.id}`,
          `address: ${counts.address}`,
          `other: ${counts.other}`,
        ];
        phiSummary.textContent = lines.join('\n');
      }

      if (phiFlags) {
        phiFlags.textContent = flags.length ? `Manual review flags: ${flags.join(', ')}` : 'Manual review flags: none';
      }

      if (statusOutput) {
        statusOutput.textContent = `Protection counts: email ${counts.email}, phone ${counts.phone}, dob ${counts.dob}, id ${counts.id}, address ${counts.address}, name ${counts.name}`;
      }
    } catch (error) {
      if (statusOutput) {
        statusOutput.textContent = 'Error';
      }
      showToast(error.message || 'Something went wrong.');
    } finally {
      setLoading(false);
    }
  };

  const handleClear = () => {
    textInput.value = '';
    if (fileInput) {
      fileInput.value = '';
    }
    if (sampleSelect) {
      sampleSelect.value = '';
    }
    if (statusOutput) {
      statusOutput.textContent = 'Ready';
    }
    if (protectedOutput) {
      protectedOutput.textContent = 'Protected output will appear here.';
    }
    if (patientOutput) {
      patientOutput.textContent = 'Run the demo to generate a patient-friendly summary.';
    }
    if (clinicianOutput) {
      clinicianOutput.textContent = 'Run the demo to generate a clinician summary.';
    }
    if (coverageOutput) {
      coverageOutput.textContent = 'Run the demo to see coverage details.';
    }
    if (phiSummary) {
      phiSummary.textContent = 'Run the demo to generate PHI scan results.';
    }
    if (phiFlags) {
      phiFlags.textContent = 'Manual review flags will appear here.';
    }
    lastClinicianViews = null;
  };

  if (placeholderSelect) {
    updateLegend(placeholderSelect.value);
    placeholderSelect.addEventListener('change', () => updateLegend(placeholderSelect.value));
  }

  if (sampleSelect) {
    sampleSelect.addEventListener('change', () => {
      const sample = sampleNotes[sampleSelect.value];
      if (sample) {
        textInput.value = sample;
      }
    });
  }

  convertButtons.forEach((btn) => {
    btn.addEventListener('click', handleConvert);
  });

  if (clearBtn) {
    clearBtn.addEventListener('click', handleClear);
  }
})();
