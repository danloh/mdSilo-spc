import {
  Button,
  ButtonGroup,
  Input,
  Popover,
  PopoverArrow,
  PopoverBody,
  PopoverCloseButton,
  PopoverContent,
  PopoverFooter,
  PopoverHeader,
  PopoverTrigger,
  useDisclosure,
} from "@chakra-ui/react";
import { useRef, useState } from "react";

type Props = {
  onNewNote: (title: string) => void;
  darkMode: boolean;
};

export default function NewNote({onNewNote, darkMode}: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const { isOpen, onOpen, onClose } = useDisclosure();
  const [title, setTitle] = useState('');
  function handleNewNote() {
    onNewNote(title);
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
              Done
            </Button>
          </ButtonGroup>
        </PopoverFooter>
      </PopoverContent>
    </Popover>
  );
}
