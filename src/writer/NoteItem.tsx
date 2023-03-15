import {
  Button,
  ButtonGroup,
  Flex,
  HStack,
  Icon,
  Input,
  Menu,
  MenuButton,
  MenuList,
  Popover,
  PopoverArrow,
  PopoverBody,
  PopoverCloseButton,
  PopoverContent,
  PopoverFooter,
  PopoverHeader,
  PopoverTrigger,
  Text,
  useDisclosure,
  IconButton,
} from "@chakra-ui/react";
import { useRef, useState } from "react";
import { VscMarkdown, VscMenu } from "react-icons/vsc";
import { SimpleNote } from "./types";

type Props = {
  note: SimpleNote;
  onOpenNote: (id: string) => void;
  onRename: (id: string, title: string) => void;
  onMoveNote: (id: string, folder: string) => void;
  onDelNote: (id: string) => void;
  darkMode: boolean;
  isActive: boolean;
};

export default function NoteItem({
  note,
  onOpenNote,
  onRename,
  onMoveNote,
  onDelNote,
  darkMode,
  isActive,
}: Props) {
  return (
    <HStack 
      justifyContent="space-between" m={1} px={2}  
      _hover={{bgColor: darkMode ? "#464647" : "gray.200"}}
      bgColor={isActive ? (darkMode ? "#464647" : "gray.300") : ""}
    >
      <HStack rounded="md" cursor="pointer">
        <Icon as={VscMarkdown} size={8} />
        <Text fontWeight="medium" onClick={() => onOpenNote(note.id)}>
          {note.title}
        </Text>
      </HStack>
      <Menu isLazy>
        <MenuButton><VscMenu size={8} /></MenuButton>
        <MenuList display='flex' flexDirection="column">
          <MoveNote note={note} onMove={onMoveNote} darkMode={darkMode} />
          <Rename note={note} onRename={onRename} darkMode={darkMode} />
          <DelNote note={note} onDelete={onDelNote} darkMode={darkMode} />
        </MenuList>
      </Menu>
    </HStack>
  );
}

type RenameProps = {
  note: any;
  onRename: (id: string, title: string) => void;
  darkMode: boolean;
};

function Rename({note, onRename, darkMode}: RenameProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const { isOpen, onOpen, onClose } = useDisclosure();
  const [title, setTitle] = useState(note.title);
  function handleRenameNote() {
    onRename(note.id, title);
    onClose();
  }

  return (
    <Popover
      placement="right"
      isOpen={isOpen}
      onClose={onClose}
      initialFocusRef={inputRef}
    >
      <PopoverTrigger>
        <Button onClick={onOpen}>Rename Note</Button>
      </PopoverTrigger>
      <PopoverContent
        bgColor={darkMode ? "#333333" : "white"}
        borderColor={darkMode ? "#464647" : "gray.200"}
      >
        <PopoverHeader
          fontWeight="semibold"
          borderColor={darkMode ? "#464647" : "gray.200"}
        >
          Rename
        </PopoverHeader>
        <PopoverArrow bgColor={darkMode ? "#333333" : "white"} />
        <PopoverCloseButton />
        <PopoverBody borderColor={darkMode ? "#464647" : "gray.200"}>
          <Input
            ref={inputRef}
            mb={2}
            value={title}
            maxLength={25}
            onChange={(event) => setTitle(event.target.value)}
          />
        </PopoverBody>
        <PopoverFooter
          display="flex"
          justifyContent="flex-end"
          borderColor={darkMode ? "#464647" : "gray.200"}
        >
          <ButtonGroup size="sm">
            <Button colorScheme="blue" onClick={handleRenameNote}>
              Rename
            </Button>
          </ButtonGroup>
        </PopoverFooter>
      </PopoverContent>
    </Popover>
  );
}

type MoveProps = {
  note: any;
  onMove: (id: string, title: string) => void;
  darkMode: boolean;
};

function MoveNote({note, onMove, darkMode}: MoveProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const { isOpen, onOpen, onClose } = useDisclosure();
  const [folder, setFolder] = useState(note.folder);
  function handleMoveNote() {
    onMove(note.id, folder);
    onClose();
  }

  return (
    <Popover
      placement="right"
      isOpen={isOpen}
      onClose={onClose}
      initialFocusRef={inputRef}
    >
      <PopoverTrigger>
        <Button onClick={onOpen}>Move Note</Button>
      </PopoverTrigger>
      <PopoverContent
        bgColor={darkMode ? "#333333" : "white"}
        borderColor={darkMode ? "#464647" : "gray.200"}
      >
        <PopoverHeader
          fontWeight="semibold"
          borderColor={darkMode ? "#464647" : "gray.200"}
        >
          Rename
        </PopoverHeader>
        <PopoverArrow bgColor={darkMode ? "#333333" : "white"} />
        <PopoverCloseButton />
        <PopoverBody borderColor={darkMode ? "#464647" : "gray.200"}>
          <Input
            ref={inputRef}
            mb={2}
            value={folder}
            maxLength={25}
            onChange={(event) => setFolder(event.target.value)}
          />
        </PopoverBody>
        <PopoverFooter
          display="flex"
          justifyContent="flex-end"
          borderColor={darkMode ? "#464647" : "gray.200"}
        >
          <ButtonGroup size="sm">
            <Button colorScheme="blue" onClick={handleMoveNote}>
              Move
            </Button>
          </ButtonGroup>
        </PopoverFooter>
      </PopoverContent>
    </Popover>
  );
}

type DelProps = {
  note: any;
  onDelete: (id: string) => void;
  darkMode: boolean;
};

function DelNote({note, onDelete, darkMode}: DelProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const { isOpen, onOpen, onClose } = useDisclosure();
  function handleDelNote() {
    onDelete(note.id);
    onClose();
  }

  return (
    <Popover
      placement="right"
      isOpen={isOpen}
      onClose={onClose}
      initialFocusRef={inputRef}
    >
      <PopoverTrigger>
        <Button onClick={onOpen}>Delete Note</Button>
      </PopoverTrigger>
      <PopoverContent
        bgColor={darkMode ? "#333333" : "white"}
        borderColor={darkMode ? "#464647" : "gray.200"}
      >
        <PopoverHeader
          fontWeight="semibold"
          borderColor={darkMode ? "#464647" : "gray.200"}
        >
          Delete
        </PopoverHeader>
        <PopoverArrow bgColor={darkMode ? "#333333" : "white"} />
        <PopoverCloseButton />
        <PopoverBody borderColor={darkMode ? "#464647" : "gray.200"}>
          <Button
            size="sm"
            w="100%"
            colorScheme={darkMode ? "whiteAlpha" : "gray"}
            onClick={handleDelNote}
          >
            Confirm Delete
          </Button>
        </PopoverBody>
      </PopoverContent>
    </Popover>
  );
}
