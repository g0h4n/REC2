rule REC2_implants
{
    meta:
        author = "g0h4n_0 <g0h4n_0@protonmail.com>"
        date_created = "2023-04-18"
        date_last_modified = "2023-04-18"
        description = "Detects REC2 implant used for external C2"
        reference = "https://github.com/g0h4n/REC2"
        hash1 = ""

    strings:
        $a1 = "Error during command execution"
        $a2 = {52 45 43 32}

        $b1 = "https://www.virustotal.com/api/v3"
        $b2 = "rec2virustotal"
        $b3 = "REC2 implant for VirusTotal"

        $c1 = "megalodon::mastodon::web_socket"
        $c2 = "rec2::modules::rec2mastodon"
        $c3 = "REC2 implant for Mastodon"

    condition:
        all of ($a*) and (
        	all of ($b*)
        	or 
        	all of ($c*)
        )
}
