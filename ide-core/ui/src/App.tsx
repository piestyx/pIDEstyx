import React, { useState, useEffect, useRef } from "react";
import Editor from "@monaco-editor/react";
import { invoke } from "@tauri-apps/api/core";
import * as monaco from "monaco-editor";

const App = () => {
  const [code, setCode] = useState("// Welcome to pIDEstyx IDE!\n");
  const [projectRoot, setProjectRoot] = useState("/path/to/project");
  const [fileList, setFileList] = useState<string[]>([]);
  const [currentFileName, setCurrentFileName] = useState("buffer.rs");

  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const editorContainerRef = useRef<HTMLDivElement>(null);
  const sidebarRef = useRef<HTMLDivElement>(null);
  const [isResizing, setIsResizing] = useState(false);

  useEffect(() => {
    refreshFileList();
    invoke<string>("load_buffer")
      .then(setCode)
      .catch(() => console.log("No existing buffer found"));
  }, [projectRoot]);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!isResizing || !sidebarRef.current) return;

      const newWidth = Math.max(150, e.clientX);
      sidebarRef.current.style.width = `${newWidth}px`;

      if (editorRef.current) {
        editorRef.current.layout();
      }
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      if (editorRef.current) editorRef.current.layout();
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseup", handleMouseUp);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing]);

  useEffect(() => {
    if (!editorContainerRef.current || !editorRef.current) return;

    const observer = new ResizeObserver(() => {
      editorRef.current!.layout();
    });

    observer.observe(editorContainerRef.current);

    return () => observer.disconnect();
  }, []);

  async function refreshFileList() {
    try {
      const files = await invoke<string[]>("list_files", { root: projectRoot });
      setFileList(files);
    } catch (err) {
      console.error("Failed to list files", err);
    }
  }

  const handleEditorChange = (value?: string) => {
    setCode(value || "");
  };

  const handleSave = async () => {
    await invoke("save_buffer", { contents: code });
  };

  const handleLoad = async () => {
    const contents = await invoke<string>("load_buffer");
    setCode(contents);
  };

  return (
    <div style={{ height: "100vh", width: "100vw", display: "flex", flexDirection: "column" }}>
      <header style={{ backgroundColor: "#222", color: "#fff", padding: "0.5rem 1rem" }}>
        pIDEstyx IDE
        <button onClick={handleSave}>Save</button>
        <button onClick={handleLoad}>Load</button>
      </header>

      <div style={{ display: "flex", flexGrow: 1, height: "100%" }}>
        <div
          ref={sidebarRef}
          style={{
            width: "250px",
            minWidth: "150px",
            backgroundColor: "#333",
            color: "#fff",
            overflowY: "auto",
            padding: "0.5rem",
            flexShrink: 0,
          }}
        >
          <div>
            Project Root:
            <input
              value={projectRoot}
              onChange={(e) => setProjectRoot(e.target.value)}
              style={{ width: "100%" }}
            />
          </div>
          {fileList.map((file) => (
            <div key={file} style={{ cursor: "pointer", padding: "0.25rem" }}>
              {file.split("/").pop()}
            </div>
          ))}
        </div>

        <div
          onMouseDown={() => setIsResizing(true)}
          style={{
            width: "5px",
            cursor: "col-resize",
            backgroundColor: "#555",
            zIndex: 10,
          }}
        />

        <div ref={editorContainerRef} style={{ flexGrow: 1, minWidth: 0 }}>
          <Editor
            height="100%"
            defaultLanguage="rust"
            theme="vs-dark"
            value={code}
            onChange={handleEditorChange}
            onMount={(editor) => {
              editorRef.current = editor;
              editor.layout();
            }}
            options={{
              fontSize: 14,
              minimap: { enabled: false },
              wordWrap: "on",
            }}
          />
        </div>
      </div>
    </div>
  );
};

export default App;
