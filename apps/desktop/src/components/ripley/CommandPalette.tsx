import { Dialog } from "@base-ui/react/dialog";
import { matchSorter } from "match-sorter";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import type { KeyboardEvent as ReactKeyboardEvent } from "react";

import { cn } from "@/lib/cn";
import { useCommandPaletteStore } from "@/store/command-palette";

import type { PaletteCommand } from "@/hooks/useCommands";

type Props = {
  commands: PaletteCommand[];
};

export function CommandPalette({ commands }: Props) {
  const open = useCommandPaletteStore((s) => s.open);
  const setOpen = useCommandPaletteStore((s) => s.setOpen);
  const recent = useCommandPaletteStore((s) => s.recent);
  const recordUse = useCommandPaletteStore((s) => s.recordUse);

  const [query, setQuery] = useState("");
  const [rawActiveIndex, setRawActiveIndex] = useState(0);
  const [prevOpen, setPrevOpen] = useState(open);
  const listId = useId();
  const inputRef = useRef<HTMLInputElement>(null);

  if (open !== prevOpen) {
    setPrevOpen(open);
    if (open) {
      setQuery("");
      setRawActiveIndex(0);
    }
  }

  useEffect(() => {
    if (open) inputRef.current?.focus();
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const handler = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, setOpen]);

  const ordered = useMemo(() => orderCommands(commands, recent, query), [commands, recent, query]);
  const activeIndex = ordered.length === 0 ? 0 : Math.min(rawActiveIndex, ordered.length - 1);

  const execute = (command: PaletteCommand) => {
    recordUse(command.id);
    setOpen(false);
    command.run();
  };

  const onInputKeyDown = (event: ReactKeyboardEvent<HTMLInputElement>) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setRawActiveIndex((idx) => Math.min(idx + 1, Math.max(0, ordered.length - 1)));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setRawActiveIndex((idx) => Math.max(0, idx - 1));
    } else if (event.key === "Enter") {
      event.preventDefault();
      const command = ordered[activeIndex];
      if (command) execute(command);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Portal>
        <Dialog.Backdrop className="fixed inset-0 z-40 bg-black/50" />
        <Dialog.Popup
          aria-label="Command palette"
          data-testid="command-palette"
          className="fixed left-1/2 top-24 z-50 w-[min(640px,92vw)] -translate-x-1/2 overflow-hidden rounded-xl border border-border-subtle bg-surface text-text-primary shadow-2xl outline-none"
        >
          <input
            ref={inputRef}
            data-testid="command-palette-input"
            aria-controls={listId}
            aria-label="Search commands"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={onInputKeyDown}
            placeholder="Type a command…"
            className="w-full border-b border-border-subtle bg-transparent px-4 py-3 text-sm outline-none placeholder:text-text-muted"
          />
          <ul
            id={listId}
            data-testid="command-palette-list"
            role="listbox"
            className="max-h-80 overflow-y-auto py-1"
          >
            {ordered.length === 0 ? (
              <li
                data-testid="command-palette-empty"
                className="px-4 py-6 text-center text-sm text-text-muted"
              >
                No commands match “{query}”.
              </li>
            ) : (
              ordered.map((command, idx) => (
                <li
                  key={command.id}
                  data-testid="command-palette-item"
                  data-command-id={command.id}
                  data-active={idx === activeIndex ? "true" : undefined}
                  role="option"
                  aria-selected={idx === activeIndex}
                >
                  <button
                    type="button"
                    onClick={() => execute(command)}
                    onMouseEnter={() => setRawActiveIndex(idx)}
                    className={cn(
                      "flex w-full items-center justify-between gap-3 px-4 py-2 text-left text-sm",
                      idx === activeIndex
                        ? "bg-surface-hover text-text-primary"
                        : "text-text-secondary hover:bg-surface-hover",
                    )}
                  >
                    <span className="flex flex-col">
                      <span className="font-medium text-text-primary">{command.label}</span>
                      {command.hint ? (
                        <span className="text-xs text-text-muted">{command.hint}</span>
                      ) : null}
                    </span>
                    <span className="text-xs uppercase tracking-wide text-text-muted">
                      {command.group}
                    </span>
                  </button>
                </li>
              ))
            )}
          </ul>
        </Dialog.Popup>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

export function orderCommands(
  commands: PaletteCommand[],
  recent: string[],
  query: string,
): PaletteCommand[] {
  const trimmed = query.trim();
  if (!trimmed) {
    const recentSet = new Set(recent);
    const recentInOrder: PaletteCommand[] = [];
    for (const id of recent) {
      const found = commands.find((c) => c.id === id);
      if (found) recentInOrder.push(found);
    }
    const rest = commands.filter((c) => !recentSet.has(c.id));
    return [...recentInOrder, ...rest];
  }
  return matchSorter(commands, trimmed, {
    keys: ["label", "hint", "keywords"],
  });
}
