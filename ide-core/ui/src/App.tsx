import React, { useState } from 'react';
import Editor from '@monaco-editor/react';
import { invoke } from '@tauri-apps/api/core';

const App = () => {
  const [code, setCode] = useState<string>("// Welcome to pIDEstyx IDE!\n");

  const handleEditorChange = (value: string | undefined) => {
    setCode(value || "");
  };

  const handleSave = async () => {
    try {
      await invoke("save_buffer", { contents: code });
      console.log("Saved successfully");
    } catch (err) {
      console.error("Failed to save", err);
    }
  };

  return (
    <div style={{ height: "100vh", width: "100vw", display: "flex", flexDirection: "column" }}>
      <header
        style={{
          backgroundColor: "#222",
          color: "#fff",
          padding: "0.5rem 1rem",
          fontSize: "1.2rem",
        }}
      >
        pIDEstyx IDE
        <button
          onClick={handleSave}
          style={{
            marginLeft: "1rem",
            padding: "0.25rem 0.5rem",
            backgroundColor: "#444",
            color: "#fff",
            border: "none",
            cursor: "pointer",
          }}
        >
          Save
        </button>
      </header>

      <div style={{ flexGrow: 1 }}>
        <Editor
          height="100%"
          defaultLanguage="rust"
          theme="vs-dark"
          value={code}
          onChange={handleEditorChange}
          options={{
            fontSize: 14,
            minimap: { enabled: false },
            wordWrap: "on",
          }}
        />
      </div>
    </div>
  );
};

export default App;