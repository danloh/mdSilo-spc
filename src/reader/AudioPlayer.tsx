import { PodType } from './types';

type Props = {
  currentPod: PodType | null;
  className?: string;
};

export default function AudioPlayer(props: Props) {
  const { currentPod, className = '' } = props;
  // console.log("current pod: ", currentPod)

  if (!currentPod) {
    return (<div className='mx-1 text-sm'>no player</div>);
  }

  return (
    <div className={`flex flex-row items-center justify-center ${className}`}>
      <audio className="ml-1 h-6" autoPlay controls src={currentPod.url} />
    </div>
  )
}
