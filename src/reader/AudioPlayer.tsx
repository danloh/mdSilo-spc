import { Box, Text } from '@chakra-ui/react';
import { PodType } from './types';

type Props = {
  currentPod: PodType | null;
  className?: string;
};

export default function AudioPlayer(props: Props) {
  const { currentPod, className = '' } = props;
  // console.log("current pod: ", currentPod)

  if (!currentPod) {
    return (<></>);
  }

  return (
    <Box mb={2} w="100%" className={`${className}`}>
      <Text fontSize="xs">{currentPod.title}</Text>
      <audio style={{height: "22px", width: "15rem"}} autoPlay controls src={currentPod.url} />
    </Box>
  )
}
