import { useEffect, useState, useRef } from "react";
import { Box, Flex, Heading, Stack, Switch } from "@chakra-ui/react";
import { preview_md } from "spc-wasm";

import { Transformer } from 'markmap-lib';
import { Markmap } from 'markmap-view/dist/index.esm';
import * as markdown from "../../../spc-server/static/css/styles.css";

type PreviewProps = {
  text: string;
  darkMode: boolean;
};

export default function Preview({ text, darkMode }: PreviewProps) {
  const [preMode, setPreMode] = useState(true); // true: preview, false: mindmap 
  function handleMode() {
    setPreMode(!preMode);
  }

  useEffect(() => {
    if (!preMode) return;
    try {
      let result = preview_md(text);
      document.getElementById("preview")!.innerHTML = result;
    } catch (error) {
      console.warn("Error on preview markdown", error);
    }
  }, [text, preMode]);

  return (
    <Stack p={1}>
      <Flex justifyContent="flex-start" w="full">
        <Heading size="xs" mx={2}>Preview</Heading>
        <Switch isChecked={!preMode} onChange={handleMode} />
        <Heading size="xs" mx={2}>MindMap</Heading>
      </Flex>
      <Box h="100%" w="100%" overflow="auto" p={3} >
        {preMode ? (
          <Box key="md-pre" __css={markdown} as="div" h="100%" id="preview" />
        ) : (
          <Mindmap text={text} />
        )}
      </Box>
    </Stack>
  );
}

type MapProps = {
  text: string;
  darkMode?: boolean;
};

function Mindmap({ text }: MapProps) {
  const transformer = new Transformer();
  // Ref for SVG element
  const refSvg = useRef<SVGSVGElement | null>(null);
  // Ref for markmap object
  const refMap = useRef();

  useEffect(() => {
    if (refMap.current) return;
    refMap.current = Markmap.create(refSvg.current);
  }, [refSvg.current]);

  useEffect(() => {
    const mm = refMap.current;
    if (!mm) return;
    const { root } = transformer.transform(text);
    (mm as any).setData(root);
    (mm as any).fit();
  }, [refMap.current, text]);

  return (
    <Box h="100vh">
      <svg style={{height: "100%", width: "100%"}} ref={refSvg} />
    </Box>
  );
}
