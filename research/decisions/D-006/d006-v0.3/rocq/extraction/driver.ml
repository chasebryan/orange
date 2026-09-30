(* D-006 v0.3, candidate C-01 (Rocq): DS-05, the standalone checker's
   command line. Trusted glue: it reads the arguments and whole files, hands
   the bytes to the extracted functions as primitive strings (the kernel's
   Pstring.of_string) and prints the primitive strings they return. Every
   verdict is computed by the extracted code.

     checker records PATH
     checker cnf OBLIGATION
     checker lrat OBLIGATION CNF_PATH CERTIFICATE_PATH

   Exit status 0 after printing a verdict or the CNF text, 1 when an input
   file cannot be read or is longer than a primitive string, 2 on bad usage
   (with the usage on standard error), 3 when the computation fails (for
   example Stack_overflow or Out_of_memory); only status 0 prints a
   verdict. *)

let usage () =
  prerr_string
    "usage: checker records PATH\n\
    \       checker cnf B-C01|B-C02\n\
    \       checker lrat B-C01|B-C02 CNF_PATH CERTIFICATE_PATH\n";
  exit 2

let obligation = function
  | "B-C01" -> Extracted.Carry_save
  | "B-C02" -> Extracted.Carry_save_unshifted
  | _ -> usage ()

let fail status message =
  prerr_endline ("checker: " ^ message);
  exit status

(* The whole file, as a primitive string. *)
let read path =
  let data =
    try
      let channel = open_in_bin path in
      let data = really_input_string channel (in_channel_length channel) in
      close_in channel;
      data
    with Sys_error message | Failure message -> fail 1 message
  in
  match Pstring.of_string data with
  | Some s -> s
  | None -> fail 1 (path ^ ": longer than a primitive string")

let print s = print_string (Pstring.to_string s)

let run () =
  match Array.to_list Sys.argv with
  | [ _; "records"; path ] -> print (Extracted.records_line (read path)); print_newline ()
  | [ _; "cnf"; name ] -> print (Extracted.cnf_text (obligation name))
  | [ _; "lrat"; name; cnf; certificate ] ->
      let o = obligation name in
      print (Extracted.lrat_line o (read cnf) (read certificate));
      print_newline ()
  | _ -> usage ()

let () = try run () with e -> fail 3 (Printexc.to_string e)
