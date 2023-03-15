import {
  Button,
  ButtonGroup,
  HStack,
  Icon,
  Input,
  Menu,
  MenuButton,
  MenuList,
  MenuItem,
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
};

export default function NoteItem({
  note,
  onOpenNote,
  onRename,
  onMoveNote,
  onDelNote,
  darkMode,
}: Props) {
  return (
    <HStack 
      p={2} rounded="md"
      _hover={{
        bgColor: darkMode ? "#464647" : "gray.200",
        cursor: "pointer",
      }}
    >
      <Icon as={VscMarkdown} />
      <Text fontWeight="medium" onClick={() => onOpenNote(note.id)}>
        {note.title}
      </Text>
      <Menu isLazy>
        <MenuButton
          as={IconButton}
          aria-label='Options'
          icon={<VscMenu />}
          variant='outline'
        />
        <MenuList>
          <MenuItem>Move</MenuItem>
          <MenuItem>
            <Rename note={note} onRename={onRename} darkMode={darkMode} />
          </MenuItem>
          <MenuItem>Delete</MenuItem>
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
  const [title, setTitle] = useState('');
  function handleNewNote() {
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
        <Button onClick={onOpen}>New Note</Button>
      </PopoverTrigger>
      <PopoverContent
        bgColor={darkMode ? "#333333" : "white"}
        borderColor={darkMode ? "#464647" : "gray.200"}
      >
        <PopoverHeader
          fontWeight="semibold"
          borderColor={darkMode ? "#464647" : "gray.200"}
        >
          New Note
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
            <Button colorScheme="blue" onClick={handleNewNote}>
              Rename
            </Button>
          </ButtonGroup>
        </PopoverFooter>
      </PopoverContent>
    </Popover>
  );
}