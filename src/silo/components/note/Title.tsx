import { memo, useEffect, useRef } from 'react';
import { useStore } from '../../lib/store';

type Props = {
  initialTitle: string;
  onChange: (value: string) => void;
  className?: string;
  isDaily?: boolean;
};

function Title(props: Props) {
  const { 
    initialTitle, 
    onChange, 
    className = '', 
    isDaily = false,
  } = props;
  const titleRef = useRef<HTMLDivElement | null>(null);

  const isCheckSpellOn = useStore((state) => state.isCheckSpellOn);
  const readMode = useStore((state) => state.readMode);

  const emitChange = () => {
    if (!titleRef.current) {
      return;
    }
    const title = titleRef.current.textContent ?? '';
    onChange(title);
  };

  // Set the initial title
  useEffect(() => {
    if (!titleRef.current) {
      return;
    }
    titleRef.current.textContent = initialTitle;
  }, [initialTitle]);

  return (
    <div
      ref={titleRef}
      className={`title text-3xl md:text-4xl font-semibold border-none focus:outline-none p-0 leading-tight cursor-text ${className}`}
      role="textbox"
      placeholder="Untitled"
      onKeyPress={(event) => {
        // Disallow newlines in the title field
        if (event.key === 'Enter') {
          event.preventDefault();
        }
      }}
      onPaste={(event) => {
        // Remove styling and newlines from the text
        event.preventDefault();
        let text = event.clipboardData.getData('text/plain');
        text = text.replace(/\r?\n|\r/g, ' ');
        document.execCommand('insertText', false, text);
      }}
      onBlur={emitChange}
      contentEditable={!(readMode || isDaily)}
      spellCheck={isCheckSpellOn}
    />
  );
}

export default memo(Title);
