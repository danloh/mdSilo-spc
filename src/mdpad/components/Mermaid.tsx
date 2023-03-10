import { useEffect, useRef } from "react";
import { Box, Stack } from "@chakra-ui/react";
import mermaid from "mermaid";

type MermaidProps = {
  text: string; 
  darkMode: boolean;
};

export default function Mermaid({ text, darkMode }: MermaidProps) {
  const config: any = {
    startOnLoad: true,
    flowchart: { htmlLabels: true },
    fontFamily: "inherit",
  };

  if (darkMode) {
    config.theme = "dark";
    config.themeVariables = {darkMode: true};
  }
  mermaid.initialize(config);
  useEffect(() => {
    try {
      mermaid.contentLoaded();
    } catch (error) {
      console.log("Error on mermaid:", error);
    }
  }, [text]);

  return (
    <Stack p={3}>
      <Box
        id="paper-mermaid"
        borderWidth="1px"
        borderColor="gray.500"
        rounded="sm"
        bgColor={darkMode ? "whiteAlpha.900" : "initial"}
      >
        <div className="mermaid">
          {text}
        </div>
      </Box>
    </Stack>
  );
}
