import { useEffect, useState } from "react";
import {
  Box, Text, Container, Flex, Heading, Select, Stack, Switch, useToast, Button,
} from "@chakra-ui/react";
import { VscEdit, VscMenu } from "react-icons/vsc";
import useStorage from "use-local-storage-state";
import { genHash } from "../useHash";
import Footer from "../mdpad/components/Footer";
import NewNote from "./NewNote";
import * as dataAgent from '../dataAgent';
import { NoteType, SimpleNote } from "./types";
import NoteItem from "./NoteItem";
import Preview from "../mdpad/components/Preview";
import { useNavigate } from "react-router-dom";

export default function NotePage() {
  const navigate = useNavigate();
  const toast = useToast();
  
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

  const [darkMode, setDarkMode] = useStorage("darkMode", () => false);
  function handleDarkMode() {
    setDarkMode(!darkMode);
  }

  async function newNote(title: string) {
    const id = `note_${genHash()}`;
    await dataAgent.newNote(id, title, '', currentFolder);
    navigate(`/app/write/${id}`);
  }

  async function openNote(id: string) {
    const note = await dataAgent.getNote(id);
    setCurrentNote(note);
    const content = note.content || '';
    setText(content);
  }

  function toEditNote() {
    if (currentNote?.id) navigate(`/app/write/${currentNote?.id}`);
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
    // getFolders(); 
    // insert locally
    const folders = [...folderList];
    folders.unshift(folder);
    setFolderList(folders);
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
        placeContent="center center"
        bgColor={darkMode ? "#575759" : "gray.200"}
      >
        <Text m={1}>{currentNote?.title || 'Taking Note'}</Text>
        <Button
          size="xs"
          mx={1}
          bgColor={darkMode ? "#575759" : "gray.200"}
          onClick={toEditNote}
        >
          <VscEdit />
        </Button>
      </Flex>
      <Flex 
        flex="1 0" 
        h="100%"
        w="100%"
        direction={{ base: 'column-reverse', md: 'row' }}
        wrap="wrap-reverse" 
        overflow="auto"
      >
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
                isActive={note.id === currentNote?.id}
              />
            ))}
          </Stack>
        </Container>
        <Box flex={1} minH="100%" h="100%" overflow="auto" key={currentNote?.id || 'blank'}>
          <Heading size="xl" m={2}>{currentNote?.title || ''}</Heading>
          <Box overflow="auto">
            <Preview text={text} darkMode={darkMode} />
          </Box>
        </Box>
      </Flex>
      <Footer />
    </Flex>
  );
}

const defaultMD: string = "Welcome";
