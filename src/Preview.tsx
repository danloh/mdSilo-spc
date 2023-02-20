import { useEffect } from "react";
import { Box } from "@chakra-ui/react";
import { preview_md } from "spc-wasm";

import * as markdown from "../spc-server/static/css/styles.css";

type PreviewProps = {
  text: string;
  darkMode: boolean;
};

function Preview({ text, darkMode }: PreviewProps) {
  useEffect(() => {
    try {
      let result = preview_md(text);
      document.getElementById("preview")!.innerHTML = result;
    } catch (error) {
      console.warn("Error on preview markdown", error);
    }
  }, [text]);

  return (
    <Box h="100%" w="100%" overflow="auto" p={3} >
      <Box __css={markdown} as="div" h="100%" id="preview" />
    </Box>
  );
}

export default Preview;
