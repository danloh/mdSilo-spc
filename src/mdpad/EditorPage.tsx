import { useEffect, useRef, useState } from "react";
import {
  Box,
  Button,
  Container,
  Flex,
  Heading,
  HStack,
  Icon,
  Input,
  InputGroup,
  InputRightElement,
  Stack,
  Switch,
  Text,
  useToast,
} from "@chakra-ui/react";
import { VscMarkdown, VscMenu, VscSave } from "react-icons/vsc";
import useStorage from "use-local-storage-state";
import { useDebounce } from "use-debounce";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import names from "./lib/bands.json";
import Pad, { UserInfo } from "./lib/mdpad";
import useHash from "../useHash";
import ConnectionStatus from "./components/ConnectionStatus";
import Footer from "./components/Footer";
import User from "./components/User";
import { MdEditor } from "./SplitEditor";

function getWsUri(id: string) {
  return (
    (window.location.origin.startsWith("https") ? "wss://" : "ws://") +
    window.location.host +
    `/api/socket/${id}`
  );
}

function generateName() {
  return names[Math.floor(Math.random() * names.length)];
}

function generateHue() {
  return Math.floor(Math.random() * 360);
}

export default function App() {
  const toast = useToast();
  const language = "markdown";
  const [connection, setConnection] = useState<
    "connected" | "disconnected" | "desynchronized"
  >("disconnected");
  const [users, setUsers] = useState<Record<number, UserInfo>>({});
  const [name, setName] = useStorage("name", generateName);
  const [hue, setHue] = useStorage("hue", generateHue);
  const [editor, setEditor] = useState<editor.IStandaloneCodeEditor>();
  const [darkMode, setDarkMode] = useStorage("darkMode", () => false);
  const pad = useRef<Pad>();
  const id = useHash();

  useEffect(() => {
    if (editor?.getModel()) {
      const model = editor.getModel()!;
      model.setValue("");
      model.setEOL(0); // LF
      pad.current = new Pad({
        uri: getWsUri(id),
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
        
        onChangeUsers: setUsers,
      });
      return () => {
        pad.current?.dispose();
        pad.current = undefined;
      };
    }
  }, [id, editor, toast, setUsers]);

  useEffect(() => {
    if (connection === "connected") {
      pad.current?.setInfo({ name, hue });
    }
  }, [connection, name, hue]);

  async function handleCopy() {
    await navigator.clipboard.writeText(`${window.location.origin}${window.location.pathname}#${id}`);
    toast({
      title: "Copied!",
      description: "Link copied to clipboard",
      status: "success",
      duration: 2000,
      isClosable: true,
    });
  }

  const [hideSide, setHideSide] = useState(false);
  function handleHideSide() {
    setHideSide(!hideSide);
  }

  function handleDarkMode() {
    setDarkMode(!darkMode);
  }

  async function handleSave() {
    const resp = await fetch(`${window.location.origin}/api/savetoarticle/${id}`);
    if (resp.ok) {
      toast({
        title: "Saved!",
        description: "Commit the change to article",
        status: "success",
        duration: 2000,
        isClosable: true,
      });
    } else {
      toast({
        title: "Failed to save!",
        description: "Something wrong happened",
        status: "error",
        duration: 1000,
        isClosable: true,
      });
    }
  }

  const [text, setText] = useState("");
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
          A Drop-in Collaborative Text Editor
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
          <ConnectionStatus darkMode={darkMode} connection={connection} />
          <Flex justifyContent="space-between" mt={4} mb={1.5} w="full">
            <Heading size="sm">Dark Mode</Heading>
            <Switch isChecked={darkMode} onChange={handleDarkMode} />
          </Flex>
          <Button
            size="sm"
            w="full"
            colorScheme={darkMode ? "whiteAlpha" : "blackAlpha"}
            borderColor={darkMode ? "green.400" : "green.600"}
            color={darkMode ? "green.400" : "green.600"}
            variant="outline"
            leftIcon={<VscSave />}
            mt={4}
            mb={1}
            onClick={handleSave}
          >
            Commit the Change
          </Button>
          <Heading mt={4} mb={1.5} size="sm">
            Share Link
          </Heading>
          <InputGroup size="sm">
            <Input
              readOnly
              pr="3.5rem"
              variant="outline"
              bgColor={darkMode ? "#3c3c3c" : "white"}
              borderColor={darkMode ? "#3c3c3c" : "white"}
              value={`${window.location.origin}${window.location.pathname}#${id}`}
            />
            <InputRightElement width="3.5rem">
              <Button
                h="1.4rem"
                size="xs"
                onClick={handleCopy}
                _hover={{ bg: darkMode ? "#575759" : "gray.200" }}
                bgColor={darkMode ? "#575759" : "gray.200"}
              >
                Copy
              </Button>
            </InputRightElement>
          </InputGroup>
          <Heading mt={4} mb={1.5} size="sm">
            Active Users
          </Heading>
          <Stack spacing={0} mb={1.5} fontSize="sm">
            <User
              info={{ name, hue }}
              isMe
              onChangeName={(name) => name.length > 0 && setName(name)}
              onChangeColor={() => setHue(generateHue())}
              darkMode={darkMode}
            />
            {Object.entries(users).map(([id, info]) => (
              <User key={id} info={info} darkMode={darkMode} />
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
            <Text>{id}</Text>
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
