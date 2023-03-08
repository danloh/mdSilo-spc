import { memo, useCallback, useRef, useState } from 'react';
import { Menu } from '@headlessui/react';
import { 
  IconCornerDownRight, IconDots, IconDotsDiagonal, IconId, IconPlus, IconTrash 
} from '@tabler/icons';
import { usePopper } from 'react-popper';
import { DropdownItem } from '../misc/Dropdown';
import Portal from '../misc/Portal';
import MoveToModal from '../note/NoteMoveModal';
import NoteMetadata from '../note/NoteMetadata';

type NoteProps = {
  noteId: string;
  className?: string;
};

const SidebarNoteLinkDropdown = (props: NoteProps) => {
  const { noteId, className } = props;

  const containerRef = useRef<HTMLButtonElement | null>(null);
  const [popperElement, setPopperElement] = useState<HTMLDivElement | null>(
    null
  );
  const { styles, attributes } = usePopper(
    containerRef.current,
    popperElement,
    { placement: 'right-start' }
  );

  const [isMoveToModalOpen, setIsMoveToModalOpen] = useState(false);
  const onMoveToClick = useCallback(() => setIsMoveToModalOpen(true), []);

  return (
    <>
      <Menu>
        {({ open }) => (
          <>
            <Menu.Button
              ref={containerRef}
              className={`rounded hover:bg-gray-300 active:bg-gray-400 dark:hover:bg-gray-600 dark:active:bg-gray-500 ${className}`}
            >
              <span className="flex items-center justify-center w-8 h-8">
                <IconDots className="text-gray-600 dark:text-gray-200" />
              </span>
            </Menu.Button>
            {open && (
              <Portal>
                <Menu.Items
                  ref={setPopperElement}
                  className="z-20 w-56 overflow-hidden bg-white rounded shadow-popover dark:bg-gray-800 focus:outline-none"
                  static
                  style={styles.popper}
                  {...attributes.popper}
                >
                  <DropdownItem onClick={onMoveToClick}>
                    <IconCornerDownRight size={18} className="mr-1" />
                    <span>Move to</span>
                  </DropdownItem>
                  <NoteMetadata noteId={noteId} />
                </Menu.Items>
              </Portal>
            )}
          </>
        )}
      </Menu>
      {isMoveToModalOpen ? (
        <Portal>
          <MoveToModal noteId={noteId} setIsOpen={setIsMoveToModalOpen} />
        </Portal>
      ) : null}
    </>
  );
};

export const SidebarNoteDropdown = memo(SidebarNoteLinkDropdown);
