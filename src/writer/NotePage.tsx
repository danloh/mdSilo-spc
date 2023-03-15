import { useEffect, useRef, useState } from "react";
import {
  Box,
  Button,
  Container,
  Flex,
  Heading,
  HStack,
  Icon,
  Select,
  Stack,
  Switch,
  Text,
  useToast,
} from "@chakra-ui/react";
import { VscMarkdown, VscMenu, VscSave } from "react-icons/vsc";
import useStorage from "use-local-storage-state";
import { useDebounce } from "use-debounce";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import Pad from "../mdpad/lib/mdpad";
import { genHash } from "../useHash";
import ConnectionStatus from "../mdpad/components/ConnectionStatus";
import Footer from "../mdpad/components/Footer";
import { MdEditor } from "../mdpad/SplitEditor";
import NewNote from "./NewNote";
import * as dataAgent from '../dataAgent';
import { NoteType, SimpleNote } from "./types";
import NoteItem from "./NoteItem";

function getWsUri(id: string) {
  return (
    (window.location.origin.startsWith("https") ? "wss://" : "ws://") +
    window.location.host +
    `/api/socket/${id}`
  );
}

export default function App() {
  const toast = useToast();
  const language = "markdown";
  const [connection, setConnection] = useState<
    "connected" | "disconnected" | "desynchronized"
  >("disconnected");
  const [editor, setEditor] = useState<editor.IStandaloneCodeEditor>();
  const [darkMode, setDarkMode] = useStorage("darkMode", () => false);
  const pad = useRef<Pad>();

  const [folderList, setFolderList] = useState<string[]>(['silo']);
  const [currentFolder, setCurrentFolder] = useState<string>('silo');
  const [noteList, setNoteList] = useState<SimpleNote[]>([]);
  const [currentNote, setCurrentNote] = useState<NoteType | null>(null);

  const getNotesByFolder = (folder: string) => {
   dataAgent.getNotesByFolder(folder).then((notes) => {
      setNoteList(notes[0]);
      setCurrentFolder(folder);
    })
  };

  const getFolders = () => {
    dataAgent.getFolders().then((folders) => {
       setFolderList(folders);
     })
   };

  useEffect(() => { 
    getNotesByFolder('silo'); 
    getFolders();
  }, []); 

  console.log("note list: ", noteList);

  useEffect(() => {
    if (!currentNote?.id) return;
    if (editor?.getModel()) {
      const model = editor.getModel()!;
      model.setValue("");
      model.setEOL(0); // LF
      pad.current = new Pad({
        uri: getWsUri(currentNote?.id),
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
  }, [currentNote?.id, editor, toast]);

  const [hideSide, setHideSide] = useState(false);
  function handleHideSide() {
    setHideSide(!hideSide);
  }

  function handleDarkMode() {
    setDarkMode(!darkMode);
  }

  async function newNote(title: string) {
    const id = `note_${genHash()}`;
    const note = await dataAgent.newNote(id, title, '');
    getNotesByFolder(currentFolder);
    setCurrentNote(note);
    setText(note.content);
  }

  async function openNote(id: string) {
    const note = await dataAgent.getNote(id);
    setCurrentNote(note);
    setText(note.content || '');
  }

  async function renameNote(id: string, title: string) {
    console.log("rename: ", id, title);
    const res = await dataAgent.renameNote(id, title);
    const notes = [...noteList];
    for (let note of notes) {
      if (note.id === res.id) {
        note.title = res.title;
        break;
      }
    }
    setNoteList(notes);
  }

  async function moveNote(id: string, folder: string) {
    console.log("move: ", id, folder)
    await dataAgent.moveNote(id, folder);
    const notes = [...noteList];
    const idx = notes.findIndex(n => n.id === id);
    notes.splice(idx, 1);
    setNoteList(notes);
    getFolders();
  }

  async function delNote(id: string) {
    console.log("del: ", id);
    await dataAgent.delNote(id);
    const notes = [...noteList];
    const idx = notes.findIndex(n => n.id === id);
    notes.splice(idx, 1);
    setNoteList(notes);
  }

  const [text, setText] = useState(defaultMD);
  const [mdString] = useDebounce(text, 100, { maxWait: 1000 });

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
        <Button
          size="xs"
          mx={1}
          bgColor={darkMode ? "#575759" : "gray.200"}
          _hover={{ bg: darkMode ? "#575759" : "gray.200" }}
          onClick={handleHideSide}
        >
          <VscMenu />
        </Button>
        <Box
          flexShrink={0}
          color={darkMode ? "#cccccc" : "#383838"}
          textAlign="center"
          fontSize="sm"
          px={2}
        >
          Writing 
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
        {!hideSide ? (
        <Container
          w="xs"
          h="100%"
          bgColor={darkMode ? "#252526" : "#f3f3f3"}
          overflowY="auto"
          lineHeight={1.4}
          py={4}
        >
          <Flex justifyContent="space-between" mt={4} mb={1.5} w="full">
            <Heading size="sm">Dark Mode</Heading>
            <Switch isChecked={darkMode} onChange={handleDarkMode} />
          </Flex>
          <NewNote onNewNote={newNote} darkMode={darkMode} />
          <Select
            size="sm"
            mt={2}
            bgColor={darkMode ? "#3c3c3c" : "white"}
            borderColor={darkMode ? "#3c3c3c" : "white"}
            value={currentFolder}
            onChange={(event) => getNotesByFolder(event.target.value)}
          >
            {folderList.map((folder) => (
              <option key={folder} value={folder} style={{ color: "black" }}>
                Folder: {folder}
              </option>
            ))}
          </Select>
          <Stack justifyItems="start" spacing={0} mb={1.5} fontSize="sm">
            {noteList.map((note) => (
              <NoteItem 
                key={note.id} 
                note={note} 
                onOpenNote={openNote}
                onRename={renameNote}
                onMoveNote={moveNote}
                onDelNote={delNote}
                darkMode={darkMode} 
              />
            ))}
          </Stack>
        </Container>) : null}
        <Flex flex={1} h="100%" minH="100%" direction="column" overflow="auto">
          <HStack
            h={6}
            spacing={1}
            color="#888888"
            fontWeight="medium"
            fontSize="13px"
            px={3.5}
            flexShrink={0}
          >
            <Icon as={VscMarkdown} fontSize="md" color="purple.500" />
            <Text>{currentNote?.title || ''}</Text>
          </HStack>
          <MdEditor 
            language={language}
            mdString={mdString}
            darkMode={darkMode}
            setText={setText}
            setEditor={setEditor}
          />
        </Flex>
      </Flex>
      <Footer />
    </Flex>
  );
}

const defaultMD: string = "Welcome";
