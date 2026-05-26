import { useState } from "react";

import { EmptyState } from "@/components/ripley/EmptyState";
import { ErrorPane } from "@/components/ripley/ErrorPane";
import { LoadingSkeleton } from "@/components/ripley/LoadingSkeleton";
import { Button } from "@/components/ui/button";
import type { SettingsDto } from "@/lib/bindings";
import { useSaveSettings, useSettings } from "@/hooks/useSettings";

const GUARD_MODES: { value: string; label: string; description: string }[] = [
  { value: "strict", label: "Strict", description: "Block any unrecognised script" },
  { value: "audit", label: "Audit", description: "Log but never block" },
  { value: "off", label: "Off", description: "Disable guard entirely" },
];

export function Settings() {
  const query = useSettings();
  const save = useSaveSettings();

  if (query.isLoading) {
    return (
      <section data-testid="settings-view" aria-busy="true" className="p-4">
        <LoadingSkeleton rows={6} />
      </section>
    );
  }
  if (query.isError) {
    return (
      <section data-testid="settings-view" className="p-4">
        <ErrorPane
          title="Settings unavailable"
          message={query.error instanceof Error ? query.error.message : String(query.error)}
        />
      </section>
    );
  }
  const settings = query.data;
  if (!settings) {
    return (
      <section data-testid="settings-view" className="p-4">
        <EmptyState title="No settings loaded" />
      </section>
    );
  }

  return (
    <SettingsForm initial={settings} onSave={(next) => save.mutate(next)} saving={save.isPending} />
  );
}

type FormProps = {
  initial: SettingsDto;
  onSave: (settings: SettingsDto) => void;
  saving: boolean;
};

function SettingsForm({ initial, onSave, saving }: FormProps) {
  const [draft, setDraft] = useState<SettingsDto>(initial);

  const update = <K extends keyof SettingsDto>(key: K, value: SettingsDto[K]) => {
    setDraft((prev) => ({ ...prev, [key]: value }));
  };

  return (
    <form
      data-testid="settings-form"
      onSubmit={(event) => {
        event.preventDefault();
        onSave(draft);
      }}
      className="flex flex-col gap-6 p-4"
    >
      <Section title="General">
        <NumberField
          label="Poll interval (seconds)"
          value={draft.poll_interval_secs}
          min={60}
          onChange={(v) => update("poll_interval_secs", v)}
          testId="poll-interval"
        />
        <Checkbox
          label="Launch at login"
          checked={draft.launch_at_login}
          onChange={(v) => update("launch_at_login", v)}
          testId="launch-at-login"
        />
      </Section>

      <Section title="Monitor">
        <Checkbox
          label="Enable monitor daemon"
          checked={draft.monitor_enabled}
          onChange={(v) => update("monitor_enabled", v)}
          testId="monitor-enabled"
        />
        <Checkbox
          label="Watch processes"
          checked={draft.monitor_watch_processes}
          onChange={(v) => update("monitor_watch_processes", v)}
          testId="monitor-watch-processes"
        />
        <Checkbox
          label="Watch persistence"
          checked={draft.monitor_watch_persistence}
          onChange={(v) => update("monitor_watch_persistence", v)}
          testId="monitor-watch-persistence"
        />
        <Checkbox
          label="Watch lockfiles"
          checked={draft.monitor_watch_lockfiles}
          onChange={(v) => update("monitor_watch_lockfiles", v)}
          testId="monitor-watch-lockfiles"
        />
      </Section>

      <Section title="Guard">
        <fieldset
          data-testid="guard-mode-group"
          aria-label="Guard mode"
          className="flex flex-col gap-2"
        >
          <legend className="text-sm font-medium text-text-primary">Mode</legend>
          {GUARD_MODES.map((mode) => {
            const id = `guard-mode-${mode.value}`;
            return (
              <div key={mode.value} className="flex items-start gap-2 text-sm">
                <input
                  id={id}
                  type="radio"
                  name="guard-mode"
                  value={mode.value}
                  data-testid={id}
                  checked={draft.guard_mode === mode.value}
                  onChange={() => update("guard_mode", mode.value)}
                />
                <label htmlFor={id}>
                  <span className="font-medium text-text-primary">{mode.label}</span>
                  <span className="block text-xs text-text-muted">{mode.description}</span>
                </label>
              </div>
            );
          })}
        </fieldset>
        <NumberField
          label="Decision timeout (seconds)"
          value={draft.guard_timeout_secs}
          min={5}
          onChange={(v) => update("guard_timeout_secs", v)}
          testId="guard-timeout"
        />
        <Checkbox
          label="Run scripts in sandbox"
          checked={draft.guard_sandbox}
          onChange={(v) => update("guard_sandbox", v)}
          testId="guard-sandbox"
        />
      </Section>

      <Section title="Posture">
        <Checkbox
          label="Strict posture"
          checked={draft.posture_strict}
          onChange={(v) => update("posture_strict", v)}
          testId="posture-strict"
        />
        <Checkbox
          label="Require lockfile"
          checked={draft.posture_require_lockfile}
          onChange={(v) => update("posture_require_lockfile", v)}
          testId="posture-require-lockfile"
        />
      </Section>

      <div className="flex items-center gap-3">
        <Button type="submit" disabled={saving} data-testid="settings-save">
          {saving ? "Saving…" : "Save"}
        </Button>
      </div>
    </form>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <fieldset className="flex flex-col gap-3 rounded-lg border border-border-subtle bg-surface p-4">
      <legend className="px-1 text-xs font-semibold uppercase tracking-wide text-text-muted">
        {title}
      </legend>
      {children}
    </fieldset>
  );
}

type NumberFieldProps = {
  label: string;
  value: number;
  min?: number;
  onChange: (value: number) => void;
  testId: string;
};

function NumberField({ label, value, min, onChange, testId }: NumberFieldProps) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-text-primary">{label}</span>
      <input
        type="number"
        value={value}
        min={min}
        data-testid={testId}
        onChange={(event) => {
          const parsed = Number(event.target.value);
          if (!Number.isNaN(parsed)) onChange(parsed);
        }}
        className="w-32 rounded-md border border-border-subtle bg-bg px-2 py-1 text-sm text-text-primary outline-none focus:border-accent"
      />
    </label>
  );
}

type CheckboxProps = {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  testId: string;
};

function Checkbox({ label, checked, onChange, testId }: CheckboxProps) {
  return (
    <label className="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={checked}
        data-testid={testId}
        onChange={(event) => onChange(event.target.checked)}
      />
      <span className="text-text-primary">{label}</span>
    </label>
  );
}
