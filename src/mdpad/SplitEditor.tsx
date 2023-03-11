import { Box } from "@chakra-ui/react";
import Editor from "@monaco-editor/react";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import Split from "react-split";
import Preview from "./components/Preview";
import Score from "./components/Score";
import Mermaid from "./components/Mermaid";
import Echarts from "./components/Echarts";

type EditorProps = {
  language: string;
  mdString: string;
  darkMode: boolean;
  setText: (text: string) => void;
  setEditor: (editor: editor.IStandaloneCodeEditor) => void
};

export default function SplitEditor(props: EditorProps) {
  const {language, mdString, darkMode, setText, setEditor} = props;

  return (
    <>
      {["markdown", "abcjs", "mermaid", "echarts"].includes(language) ? (
        <Box flex={1} minH={0} h="100%" >
          <Split className="split" minSize={50}>
            <Box>
              <Editor
                theme={darkMode ? "vs-dark" : "vs"}
                language={language}
                options={{
                  automaticLayout: true,
                  fontSize: 14,
                  wordWrap: "on",
                }}
                onMount={(editor) => setEditor(editor)}
                onChange={(text) => {
                  if (text !== undefined) {
                    setText(text);
                  }
                }}
              />
            </Box>
            <Box overflow="auto">
              {language === "markdown" ? (
                <Preview text={mdString} darkMode={darkMode} />
              ) : language === "abcjs" ? (
                <Score notes={mdString} darkMode={darkMode} />
              ) : language === "echarts" ? (
                <Echarts key={mdString.length} text={mdString} darkMode={darkMode} />
              ) : (
                <Mermaid key={mdString.length} text={mdString} darkMode={darkMode} />
              )}
            </Box>
          </Split>
        </Box>
      ) : (
        <Box flex={1} minH={0}>
          <Editor
            theme={darkMode ? "vs-dark" : "vs"}
            language={language}
            options={{
              automaticLayout: true,
              fontSize: 14,
              wordWrap: "on",
            }}
            onMount={(editor) => setEditor(editor)}
          />
        </Box>
      )}
    </> 
  );
}
