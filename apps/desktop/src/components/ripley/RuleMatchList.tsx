type Props = {
  rules: string[];
};

export function RuleMatchList({ rules }: Props) {
  if (rules.length === 0) {
    return (
      <p className="text-text-muted text-sm italic" data-testid="no-rules">
        No matched rules
      </p>
    );
  }
  return (
    <ul
      className="space-y-1 text-sm text-text-secondary"
      data-testid="rule-list"
      aria-label="Matched detection rules"
    >
      {rules.map((rule) => (
        <li
          key={rule}
          className="rounded border border-border-subtle bg-surface px-2 py-1 font-mono"
        >
          {rule}
        </li>
      ))}
    </ul>
  );
}
