import { HStack, Icon, Text } from "@chakra-ui/react";
import { VscCircleFilled } from "react-icons/vsc";

type StatusProps = {
  connection: "connected" | "disconnected" | "desynchronized";
  darkMode: boolean;
};

export default function ConnectionStatus({ connection, darkMode }: StatusProps) {
  return (
    <HStack spacing={1} mr={6}>
      <Icon
        as={VscCircleFilled}
        color={
          {
            connected: "green.500",
            disconnected: "orange.500",
            desynchronized: "red.500",
          }[connection]
        }
      />
      <Text
        fontSize="sm"
        fontStyle="italic"
        color={darkMode ? "gray.300" : "gray.600"}
      >
        {
          {
            connected: "Connected!",
            disconnected: "Connecting to the server...",
            desynchronized: "Disconnected, please refresh.",
          }[connection]
        }
      </Text>
    </HStack>
  );
}
