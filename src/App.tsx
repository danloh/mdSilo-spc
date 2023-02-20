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
  Link,
  Select,
  Stack,
  Switch,
  Text,
  useToast,
} from "@chakra-ui/react";
import {
  VscChevronRight,
  VscFolderOpened,
  VscGist,
  VscRepoPull,
  VscSave,
} from "react-icons/vsc";
import useStorage from "use-local-storage-state";
import { useDebounce } from "use-debounce";
import Editor from "@monaco-editor/react";
import { editor } from "monaco-editor/esm/vs/editor/editor.api";
import Split from "react-split";
import padRaw from "../spc-server/src/pad/mdpad.rs?raw";
import sample from "../README.md?raw";
import languages from "./lib/languages.json";
import animals from "./lib/animals.json";
import Pad, { UserInfo } from "./lib/mdpad";
import useHash from "./useHash";
import ConnectionStatus from "./ConnectionStatus";
import Footer from "./Footer";
import User from "./User";
import Preview from "./Preview";
import Score from "./Score";
import "./split.css";

function getWsUri(id: string) {
  return (
    (window.location.origin.startsWith("https") ? "wss://" : "ws://") +
    window.location.host +
    `/api/socket/${id}`
  );
}

function generateName() {
  return "Anonymous " + animals[Math.floor(Math.random() * animals.length)];
}

function generateHue() {
  return Math.floor(Math.random() * 360);
}

function App() {
  const toast = useToast();
  const [language, setLanguage] = useState("markdown");
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
        onChangeLanguage: (language) => {
          if (languages.includes(language)) {
            setLanguage(language);
          }
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

  function handleChangeLanguage(language: string) {
    setLanguage(language);
    if (pad.current?.setLanguage(language)) {
      toast({
        title: "Language updated",
        description: (
          <>
            All users are now editing in{" "}
            <Text as="span" fontWeight="semibold">
              {language}
            </Text>
            .
          </>
        ),
        status: "info",
        duration: 2000,
        isClosable: true,
      });
    }
  }

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

  function handleLoadSample() {
    const samples = [sample, padRaw];
    const idx = Math.floor(Math.random() * samples.length);

    if (editor?.getModel()) {
      const model = editor.getModel()!;
      model.pushEditOperations(
        editor.getSelections(),
        [
          {
            range: model.getFullModelRange(),
            text: samples[idx],
          },
        ],
        () => null
      );
      editor.setPosition({ column: 0, lineNumber: 0 });
      const lang = idx === 0 ? "markdown" : "rust";
      handleChangeLanguage(lang);
    }
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
      <Box
        flexShrink={0}
        bgColor={darkMode ? "#333333" : "#e8e8e8"}
        color={darkMode ? "#cccccc" : "#383838"}
        textAlign="center"
        fontSize="sm"
        py={0.5}
      >
        mdpad | Collaborative Text Editor
      </Box>
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
          <ConnectionStatus darkMode={darkMode} connection={connection} />
          <Flex justifyContent="space-between" mt={4} mb={1.5} w="full">
            <Heading size="sm">Dark Mode</Heading>
            <Switch isChecked={darkMode} onChange={handleDarkMode} />
          </Flex>
          <Heading mt={4} mb={1.5} size="sm">
            Language
          </Heading>
          <Select
            size="sm"
            bgColor={darkMode ? "#3c3c3c" : "white"}
            borderColor={darkMode ? "#3c3c3c" : "white"}
            value={language}
            onChange={(event) => handleChangeLanguage(event.target.value)}
          >
            {languages.map((lang) => (
              <option key={lang} value={lang} style={{ color: "black" }}>
                {lang}
              </option>
            ))}
          </Select>
          <Button
            size="sm"
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
          <Text fontSize="sm" my={1.5}>
            Share the link for live collaboration.
          </Text>
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
          <Button
            size="sm"
            colorScheme={darkMode ? "whiteAlpha" : "blackAlpha"}
            borderColor={darkMode ? "purple.400" : "purple.600"}
            color={darkMode ? "purple.400" : "purple.600"}
            variant="outline"
            leftIcon={<VscRepoPull />}
            mt={1}
            onClick={handleLoadSample}
          >
            Load an example
          </Button>
        </Container>
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
            <Icon as={VscFolderOpened} fontSize="md" color="blue.500" />
            <Text>documents</Text>
            <Icon as={VscChevronRight} fontSize="md" />
            <Icon as={VscGist} fontSize="md" color="purple.500" />
            <Text>{id}</Text>
          </HStack>
          {language === "markdown" || language === "abcjs" ? (
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
                  ) : (
                    <Score notes={mdString} darkMode={darkMode} />
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
        </Flex>
      </Flex>
      <Footer />
    </Flex>
  );
}

export default App;
