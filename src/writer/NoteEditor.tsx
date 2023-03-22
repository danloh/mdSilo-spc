import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router-dom";
import { Box, Flex, Switch, useToast } from "@chakra-ui/react";
import useStorage from "use-local-storage-state";
import { useDebounce } from "use-debounce";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import Pad from "../mdpad/lib/mdpad";
import ConnectionStatus from "../mdpad/components/ConnectionStatus";
import Footer from "../mdpad/components/Footer";
import { MdEditor } from "../mdpad/SplitEditor";
import * as dataAgent from '../dataAgent';
import { NoteType } from "./types";

function getWsUri(id: string) {
  return (
    (window.location.origin.startsWith("https") ? "wss://" : "ws://") +
    window.location.host +
    `/api/socket/${id}`
  );
}

export default function NoteEditor() {
  const toast = useToast();
  const language = "markdown";
  const [connection, setConnection] = useState<
    "connected" | "disconnected" | "desynchronized"
  >("disconnected");
  const [editor, setEditor] = useState<editor.IStandaloneCodeEditor>();
  const pad = useRef<Pad>();
  const { id } = useParams<string>();
  const [currentNote, setCurrentNote] = useState<NoteType | null>(null);

  function getNote(id: string) {
    dataAgent.getNote(id).then(note => setCurrentNote(note));
  }

  useEffect(() => { getNote(id!); }, [id]); 

  useEffect(() => {
    if (editor?.getModel()) {
      const model = editor.getModel()!;
      model.setValue("");
      model.setEOL(0); // LF
      pad.current = new Pad({
        uri: getWsUri(id!),
        editor,
        onConnected: () => setConnection("connected"),
        onDisconnected: () => setConnection("disconnected"),
        onDesynchronized: () => {
          setConnection("desynchronized");
          toast({
            title: "Desynchronized with server",
            description: "Please save your work and refresh the page.",
            status: "error",
            duration: null,
          });
        },
      });
      return () => {
        pad.current?.dispose();
        pad.current = undefined;
      };
    }
  }, [id, editor, toast]);

  const [darkMode, setDarkMode] = useStorage("darkMode", () => false);
  function handleDarkMode() {
    setDarkMode(!darkMode);
  }

  const [text, setText] = useState(defaultMD);
  const [mdString] = useDebounce(text, 10, { maxWait: 100 });

  return (
    <Flex
      direction="column"
      h="100vh"
      overflow="hidden"
      bgColor={darkMode ? "#1e1e1e" : "white"}
      color={darkMode ? "#cbcaca" : "inherit"}
    >
      <Flex
        direction="row"
        w="100%"
        overflow="hidden"
        placeContent="center space-between"
        bgColor={darkMode ? "#575759" : "gray.200"}
      >
        <Switch m={1} isChecked={darkMode} onChange={handleDarkMode} />
        <Box
          flexShrink={0}
          color={darkMode ? "#cccccc" : "#383838"}
          textAlign="center"
          fontSize="sm"
          px={2}
        >
          {currentNote?.title} 
        </Box>
        <ConnectionStatus darkMode={darkMode} connection={connection} />
      </Flex>
      <Flex 
        flex="1 0" 
        h="100%"
        w="100%"
        direction={{ base: 'column-reverse', md: 'row' }}
        wrap="wrap-reverse" 
        overflow="auto"
      >
        <MdEditor 
          key={currentNote?.id || 'blank'}
          language={language}
          mdString={mdString}
          darkMode={darkMode}
          setText={setText}
          setEditor={setEditor}
          wikiBase={"app/notes"}
          tagBase={"app/tag"}
        />
      </Flex>
      <Footer />
    </Flex>
  );
}

const defaultMD: string = "Welcome";
