import { useEffect, useRef } from "react";
import { Box, Stack } from "@chakra-ui/react";
import * as echarts from "echarts";

type EchartsProps = {
  text: string; 
  darkMode: boolean;
};

export default function Echarts({ text, darkMode }: EchartsProps) {
  useEffect(() => {
    try {
      const wraper = document.getElementById("paper-echarts");
      if (wraper) {
        const echartsData = JSON.parse(text);
        // console.log("chart data", echartsData, text)
        echarts.init(
          wraper, 
          darkMode ? "dark" : undefined,
          { renderer: 'svg',  height: 600, }
        )
        .setOption(echartsData);
      }
    } catch (error) {
      console.log("Error on Echartsjs:", error);
    }
  }, [text, darkMode]);

  return (
    <Stack p={3}>
      <Box
        id="paper-echarts"
        borderWidth="1px"
        borderColor="gray.500"
        rounded="sm"
        bgColor={darkMode ? "whiteAlpha.900" : "initial"}
      />
    </Stack>
  );
}
