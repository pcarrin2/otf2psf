#!/usr/bin/env python3
from unicode_charnames import charname
import unicodeblocks
from prompt_toolkit.shortcuts import yes_no_dialog, input_dialog, radiolist_dialog, message_dialog, checkboxlist_dialog
from time import sleep

# apologies for the ugly source. this isn't meant to be load-bearing software

def char_display(character):
    # formats charset file lines for a character.
    if character is None or len(character) != 1:
        return "U+0000\n# Filler character\n"
    else:
        return "U+{:04x}\n# U+{:04x}:\t{}\n".format(ord(character), ord(character), charname(character))

def is_printable(character):
    # unicode_charnames.charname() will return a code wrapped in "<>" for non-printable characters.
    # kludgey I know.
    return charname(character)[0] != "<"

def write_list(characters):
    # writes charset to file in the correct format
    filename = input_dialog(title='Write charset to file', text='Destination file: ', default='char.set').run()
    table = "".join([char_display(c) for c in characters])
    with open(filename, 'w') as f:
        f.write(table)
    message_dialog(title='Done', text=f'Wrote charset to {filename}.').run()

def add_chars(characters, to_add):
    # adds a list of Unicode characters (to_add, list of strings of length 1) to existing charset (`characters`, list of either strings of length 1 or None).
    # changes the value of `characters`.
    couldnt_fit = []
    already_present = []
    for c in to_add:
        if c in characters:
            already_present.append(c)
        elif is_printable(c):
            # None represents empty slots in the charset; if there are no Nones left then the charset is full and subsequent characters can't fit.
            if None in characters:
               index = characters.index(None)
               characters[index] = c
            else:
                couldnt_fit.append(c)

    if couldnt_fit:
        formatted_list = "\n".join([c + "    " + charname(c) for c in couldnt_fit])
        message_dialog(title='Warning', text=f"{len(couldnt_fit)} selected characters didn't fit in your character set. They include:\n{formatted_list}").run()

    if already_present:
        formatted_list = "\n".join([c + "    " + charname(c) for c in already_present])
        message_dialog(title='Warning', text=f"{len(already_present)} selected characters didn't fit in your character set. They include:\n{formatted_list}").run()

def add_ascii(characters):
    # like add_chars, but it preserves the indices of ASCII characters, so the 40th character in the charset is the 40th ASCII character for example.
    # only adds printable ASCII characters. also assumes that `characters` hasn't been populated at all so far.
    # changes the value of `characters`.
    couldnt_fit = []
    printable_ascii = [chr(i) for i in range(32, 127)]
    for c in printable_ascii:
        index = ord(c)
        if index >= len(characters):
            couldnt_fit.append(c)
        else:
            characters[index] = c
    if couldnt_fit:
        message_dialog(title='Warning', text=f"{len(couldnt_fit)} printable ASCII characters didn't fit in your charset. Please select a font size of at least 126 characters to avoid this.").run()


def get_char_count():
    # ask the user how many characters should be in the charset
    dialog_text = "Number of characters in charset: "
    while True:
        char_count_str = input_dialog(title='Character count', text=dialog_text).run()
        if char_count_str is None:
            return None
        try:
            char_count = int(char_count_str)
            return char_count
        except:
            dialog_text = "Your input could not be parsed as an integer. Try again.\nNumber of characters in charset:"

def main():
    char_count = get_char_count()
    if char_count is None:
        exit()

    # Charset
    characters = [None] * char_count

    include_ascii = yes_no_dialog(title='Add ASCII characters?', text='Add ASCII characters to character set?').run()
    if include_ascii:
        add_ascii(characters)

    # setting up variables for main loop
    # common_blocks chosen purely based on my preference. I am very anglophone but I see quite a few PSF fonts including Cyrillic characters so I put those in
    common_blocks = ['latin1supplement', 'latinextendeda', 'latinextendedb', 'cyrillic', 'cyrillicsupplement', 'currencysymbols', 'arrows', 'boxdrawing', 'blockelements', 'geometricshapes', 'miscellaneoussymbols'] 
    block_choices = [(b, unicodeblocks.blocks.get(b).name) for b in common_blocks]
    block_choices.append(('OTHER', 'Choose a different Unicode block...'))
    
    menu_choices = [
        ('BLOCK', 'Add characters from a Unicode block'),
        ('CHAR', 'Add an individual character'),
        ('EDIT', 'View/edit current charset'),
        ('SAVE', 'Save charset to a file'),
        ('CANCEL', 'Quit without saving'),
        ]

    while True:
        free_entries = characters.count(None)
        choice = radiolist_dialog(title='Main menu', text=f'{free_entries} entries left in character set.\nChoose an action:', values=menu_choices).run()
        match choice:
            case 'BLOCK':
                block_raw = radiolist_dialog(
                        title='Unicode block selection',
                        text='Pick a Unicode block to add characters from.',
                        values=block_choices
                        ).run()

                if block_raw == 'OTHER':
                    all_blocks = [(b, unicodeblocks.blocks.get(b).name) for b in unicodeblocks.blocks.keys()]
                    block_raw = radiolist_dialog(
                        title='Unicode block selection',
                        text='Pick a Unicode block to add characters from.',
                        values=all_blocks
                        ).run()
                elif block_raw is None:
                    continue

                block = unicodeblocks.blocks.get(block_raw)

                char_select_values = [(chr(c), chr(c) + "    " + charname(chr(c))) for c in range(block.start, block.end) if is_printable(chr(c))]
                char_select_values.insert(0, ("ALL", "Add all characters from block"))
                to_add = checkboxlist_dialog(
                        title='Add characters from Unicode block',
                        text='Choose characters to add.',
                        values=char_select_values
                        ).run()
                
                if to_add is None:
                    continue

                if 'ALL' in to_add:
                    to_add = [chr(c) for c in range(block.start, block.end) if is_printable(chr(c))]

                add_chars(characters, to_add)

            case 'CHAR':
                char_to_add = None
                while char_to_add is None:
                    char_response = input_dialog(
                            title='Add single character', 
                            text='Paste a single Unicode character, or type a codepoint as "U+[hex]".\nPlease note that multi-character sequences are not supported at this time.'
                            ).run()
                    
                    if char_response is None:
                        # this only happens if they selected the "cancel" button -- break out of this branch and send them back to the main menu
                        break

                    match list(char_response):
                        # one-character response: add this character to the charset
                        case [c]:
                            char_to_add = c
                        # multi-character response beginning in "u+": interpret as unicode codepoint
                        case ["u", "+", *codepoint] | ["U", "+", *codepoint]:
                            try:
                                char_to_add = chr(int("".join(codepoint), 16))
                            except ValueError:
                                message_dialog(title="Error", text="Sorry, I couldn't parse your Unicode codepoint as a hexadecimal number. Check what you typed and try again.").run()
                        case _:
                            # multi-char responses including ones with combining diacritical marks will fall into this bucket
                            message_dialog(title="Error", text="Sorry, I couldn't parse your input as a Unicode character. You may have better luck writing it out as U+[hex].").run()

                if char_to_add is None:
                    # if they selected the "cancel" button
                    continue

                add_chars(characters, [char_to_add])

            case 'EDIT':
                char_choices = [(i, characters[i] + "    " + charname(characters[i])) for i in range(len(characters)) if characters[i] is not None]
                if len(char_choices) == 0:
                    message_dialog(title="No characters selected", text="You haven't added any characters to your charset yet. Please add some first!").run()
                else:
                    to_delete = checkboxlist_dialog(title="View/edit character set", text="Select entries from the list to delete them:", values=char_choices).run()
                    if to_delete:
                        if yes_no_dialog(title="Confirm deletion", text="Delete these characters?").run():
                            for i in to_delete:
                                characters[i] = None

            case 'SAVE':
                write_list(characters)
                exit()
            case 'CANCEL' | None:
                exit()


if __name__ == '__main__':
    main()
