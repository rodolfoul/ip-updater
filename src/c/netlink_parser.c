#include <arpa/inet.h>
#include <linux/rtnetlink.h>
#include <net/if.h>
#include <stdio.h>

enum Action
{
	UNDEFINED,
	ADD,
	DEL
};

struct IfMessage {
	char interface_name[IF_NAMESIZE];
	char related_address[INET6_ADDRSTRLEN];
	enum Action action;
};

void parse_message(char *buf, int buf_len, struct IfMessage *result)
{
	const struct nlmsghdr *nl_msg_header = (struct nlmsghdr *)buf;
	int nl_msg_len = nl_msg_header->nlmsg_len;
	struct ifaddrmsg *interface_data = (struct ifaddrmsg *)NLMSG_DATA(nl_msg_header);

	struct in6_addr *addrp;
	struct rtattr *route_iterator = IFA_RTA(interface_data);
	while (RTA_OK(route_iterator, nl_msg_len)) {
		if (route_iterator->rta_type == IFA_ADDRESS) {
			addrp = ((struct in6_addr *)(route_iterator + 1));
			break;
		}
		route_iterator = RTA_NEXT(route_iterator, nl_msg_len);
	}
	inet_ntop(AF_INET6, addrp, result->related_address, sizeof(result->related_address)); // get IP addr
	if_indextoname(interface_data->ifa_index, result->interface_name);
	if (nl_msg_header->nlmsg_type == RTM_NEWADDR) {
		result->action = ADD;
	}
	else if (nl_msg_header->nlmsg_type == RTM_DELADDR) {
		result->action = DEL;
	}
	else {
		result->action = UNDEFINED;
	}

	buf_len -= NLMSG_ALIGN(nl_msg_len); // Check if we have more messages to parse
	if (buf_len >= (ssize_t)sizeof(*nl_msg_header)) {
		printf("WARN: Messages not parsed while analyzing buffer");
	}
}