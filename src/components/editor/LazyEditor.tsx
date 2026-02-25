/**
 * LazyEditor — wrapper that only mounts Monaco editor for the active tab.
 *
 * Background tabs have their Monaco model preserved but the editor DOM
 * is unmounted to reduce memory pressure and DOM node count.
 *
 * Usage in EditorGroupPanel (MultiBuffer.tsx):
 *   <LazyEditor file={file} isActive={isActive} groupId={groupId} />
 */

import { Show, createSignal, createEffect, onCleanup, lazy, Suspense, ErrorBoundary } from "solid-js";
import type { OpenFile } from "@/context/EditorContext";
import { EditorSkeleton } from "./EditorSkeleton";

const CodeEditorLazy = lazy(() =>
  import("./CodeEditor").then((m) => ({ default: m.CodeEditor })),
);

export interface LazyEditorProps {
  file: OpenFile;
  isActive: boolean;
  groupId: string;
}

const mountedModels = new Set<string>();

export function LazyEditor(props: LazyEditorProps) {
  const [wasEverActive, setWasEverActive] = createSignal(props.isActive);

  createEffect(() => {
    if (props.isActive) {
      setWasEverActive(true);
      mountedModels.add(props.file.id);
    }
  });

  onCleanup(() => {
    // Model cleanup happens in CodeEditor's own onCleanup
  });

  return (
    <div
      style={{
        display: props.isActive ? "flex" : "none",
        flex: "1",
        "min-height": "0",
        "flex-direction": "column",
        overflow: "hidden",
      }}
      data-lazy-editor={props.file.id}
      data-active={props.isActive}
    >
      <Show when={wasEverActive()}>
        <ErrorBoundary fallback={(err, reset) => (
          <div style={{
            flex: "1",
            display: "flex",
            "flex-direction": "column",
            "align-items": "center",
            "justify-content": "center",
            gap: "12px",
            background: "var(--jb-panel, #1e1e1e)",
            color: "var(--jb-text-primary, #ccc)",
            padding: "24px",
          }}>
            <div style={{ "font-size": "14px", "font-weight": "600" }}>
              Failed to load editor for {props.file.name}
            </div>
            <div style={{ "font-size": "12px", color: "var(--jb-text-muted-color, #888)", "max-width": "400px", "text-align": "center" }}>
              {err?.message || "Unknown error"}
            </div>
            <button
              onClick={reset}
              style={{
                padding: "6px 16px",
                background: "var(--jb-interactive-default, #0e639c)",
                color: "#fff",
                border: "none",
                "border-radius": "4px",
                cursor: "pointer",
                "font-size": "12px",
              }}
            >
              Retry
            </button>
          </div>
        )}>
          <Suspense fallback={<EditorSkeleton />}>
            <CodeEditorLazy file={props.file} groupId={props.groupId} />
          </Suspense>
        </ErrorBoundary>
      </Show>
    </div>
  );
}

export function hasEditorModel(fileId: string): boolean {
  return mountedModels.has(fileId);
}

export default LazyEditor;
