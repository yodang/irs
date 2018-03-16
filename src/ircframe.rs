use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command
{
    PASS,
    NICK,
    USER,
    SERVER,
    OPER,
    QUIT,
    SQUIT,
    JOIN,
    PART,
    MODE,
    TOPIC,
    NAMES,
    LIST,
    INVITE,
    KICK,
    VERSION,
    STATS,
    LINKS,
    TIME,
    CONNECT,
    TRACE,
    ADMIN,
    INFO,
    PRIVMSG,
    NOTICE,
    WHO,
    WHOIS,
    WHOWAS,
    KILL,
    PING,
    PONG,
    ERROR,
    AWAY,
    Reply(u16)
}

#[derive(Debug)]
pub struct IrcFrame(Command, Vec<String>);


fn parse_command(s: &str) -> Result<Command, ()>
{
    match s
    {
        "PASS" => Ok(Command::PASS),
        "NICK" => Ok(Command::NICK),
        "USER" => Ok(Command::USER),
        "SERVER" => Ok(Command::SERVER),
        "OPER" => Ok(Command::OPER),
        "QUIT" => Ok(Command::QUIT),
        "SQUIT" => Ok(Command::SQUIT),
        "JOIN" => Ok(Command::JOIN),
        "PART" => Ok(Command::PART),
        "MODE" => Ok(Command::MODE),
        "TOPIC" => Ok(Command::TOPIC),
        "NAMES" => Ok(Command::NAMES),
        "LIST" => Ok(Command::LIST),
        "INVITE" => Ok(Command::INVITE),
        "KICK" => Ok(Command::KICK),
        "VERSION" => Ok(Command::VERSION),
        "STATS" => Ok(Command::STATS),
        "LINKS" => Ok(Command::LINKS),
        "TIME" => Ok(Command::TIME),
        "CONNECT" => Ok(Command::CONNECT),
        "TRACE" => Ok(Command::TRACE),
        "ADMIN" => Ok(Command::ADMIN),
        "INFO" => Ok(Command::INFO),
        "PRIVMSG" => Ok(Command::PRIVMSG),
        "NOTICE" => Ok(Command::NOTICE),
        "WHO" => Ok(Command::WHO),
        "WHOIS" => Ok(Command::WHOIS),
        "WHOWAS" => Ok(Command::WHOWAS),
        "KILL" => Ok(Command::KILL),
        "PING" => Ok(Command::PING),
        "PONG" => Ok(Command::PONG),
        "ERROR" => Ok(Command::ERROR),
        "AWAY" => Ok(Command::AWAY),
        _ =>
        {
            let code=s.parse::<u16>();
            if code.is_ok()
            {
                Ok(Command::Reply(code.unwrap()))
            }
            else
            {
                Err(())
            }
        }
    }
}

impl FromStr for IrcFrame
{
    type Err=();
    fn from_str(s: &str) -> Result<Self, Self::Err>
    {
        //println!("Parse: {}", s);
        let mut iter=s.split(" ");
        let first=iter.nth(0).unwrap();
        // Extract command
        let command=if !first.starts_with(":")
        {
            parse_command(first)
        }
        else
        {
            parse_command(iter.nth(0).unwrap())
        };
        
        // Extract params
        let params: Vec<String>=iter.map(|s|s.to_owned()).collect::<Vec<String>>().iter()
        //Fold back params after the first colon
        .fold(Vec::new(),|mut v,e|
        {
            let mut push_e=true;
            if let Some(s)=v.last_mut()
            {
                if s.starts_with(":")
                {
                    s.push_str(&format!(" {}",&e));
                    push_e=false;
                }
            }
            if push_e
            {
                v.push(e.clone());
            }
            v
        });

        match command
        {
            Ok(c) => Ok(IrcFrame(c, params)),
            Err(_) => Err(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ping()
    {
        assert_eq!(parse_command("PING"), Ok(Command::PING));
    }

    #[test]
    fn parse_reply()
    {
        assert_eq!(parse_command("381"), Ok(Command::Reply(381)));
    }

    #[test]
    fn parse_error()
    {
        assert_eq!(parse_command("PING "), Err(()));
    }

}
